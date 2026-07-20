use std::sync::Arc;

use cornetti::{
    actix::auth::helpers::invalidate_session,
    auth::{
        confs::JwtAuthConf,
        models::{AuthorizationPermission, JwtDefaultClaims},
        traits::{IdentityAuthorization, SessionStore},
    },
    core::{errors, models::CornettiResult},
    mongo::services::MongoDBService,
    redis::services::RedisDBService,
};

use crate::base::{
    identity::repos::IdentityRepository,
    users::repos::{UsersCacheRepository, UsersRepository},
};

pub struct AuthenticationService<'a, S: SessionStore> {
    users_repository: UsersRepository,
    identity_repository: IdentityRepository,
    conf: &'a JwtAuthConf,
    tenant_id: &'a str,
    session_store: Option<Arc<S>>,
}

impl<'a, S: SessionStore> AuthenticationService<'a, S> {
    pub fn new(
        mongo: Arc<MongoDBService>,
        conf: &'a JwtAuthConf,
        tenant_id: &'a str,
        session_store: Option<Arc<S>>,
    ) -> AuthenticationService<'a, S> {
        AuthenticationService {
            users_repository: UsersRepository::new(mongo.clone()),
            identity_repository: IdentityRepository::new(mongo),
            conf,
            tenant_id,
            session_store,
        }
    }

    pub async fn login(
        &self,
        login: cornetti::auth::models::DefaultLoginBody,
        req: actix_web::HttpRequest,
    ) -> CornettiResult<(
        cornetti::auth::models::DefaultLoginResponse<crate::base::users::models::User>,
        Option<actix_web::cookie::Cookie<'_>>,
        Option<actix_web::cookie::Cookie<'_>>,
        Option<actix_web::cookie::Cookie<'_>>,
        Option<actix_web::cookie::Cookie<'_>>,
    )> {
        let _ = self
            .users_repository
            .get_by_user_password(&login.username, &login.password)
            .await?;

        let user = self
            .users_repository
            .set_last_access(&login.username)
            .await?;

        cornetti::actix::auth::helpers::generate_auth_tokens_and_response(
            &self.conf,
            user,
            login.username,
            self.tenant_id,
            req,
            self.session_store.clone(),
        )
        .await
    }

    pub async fn logout(&self, claims: Option<JwtDefaultClaims>) -> CornettiResult<Vec<&str>> {
        if let Some(c) = claims {
            let _ = self.identity_repository.get_user_by_email(&c.sub).await?;

            return invalidate_session(
                self.conf,
                self.session_store.clone(),
                c.sub,
                c.session_id,
                self.tenant_id,
            )
            .await;
        }

        Err(errors::not_found::item_not_found())
    }

    pub async fn refresh(
        &self,
        claims: Option<JwtDefaultClaims>,
        req: actix_web::HttpRequest,
    ) -> CornettiResult<(
        cornetti::auth::models::RefreshAuthResponse<crate::base::identity::models::UserIdentity>,
        Option<actix_web::cookie::Cookie<'_>>,
        Option<actix_web::cookie::Cookie<'_>>,
    )> {
        if let Some(c) = claims {
            let user = self.identity_repository.get_identity(&c.sub).await?;
            return cornetti::actix::auth::helpers::refresh_auth_tokens_and_response(
                &self.conf,
                user,
                c,
                self.tenant_id,
                req,
                self.session_store.clone(),
            )
            .await;
        }

        Err(errors::not_found::item_not_found())
    }
}

pub struct UserAuthorizationService {
    repository: UsersRepository,
    cache_repository: UsersCacheRepository,
    app_namespace: String,
}

impl UserAuthorizationService {
    pub fn new(
        mongo: Arc<MongoDBService>,
        redis: Arc<RedisDBService>,
        app_namespace: String,
    ) -> Self {
        Self {
            repository: UsersRepository::new(mongo),
            cache_repository: UsersCacheRepository::new(redis),
            app_namespace,
        }
    }
}

impl IdentityAuthorization for UserAuthorizationService {
    fn get_identity_permissions(
        &self,
        _tenant_id: &str,
        sub: &str,
    ) -> impl std::future::Future<
        Output = CornettiResult<std::collections::HashMap<String, AuthorizationPermission>>,
    > + Send {
        Box::pin(async move {
            let cached = self
                .cache_repository
                .get_identity_permissions(&self.app_namespace, sub)
                .await?;

            match cached {
                Some(permissions) => Ok(permissions),
                None => {
                    let permissions = self.repository.get_user_permissions(sub).await?;
                    self.cache_repository
                        .set_identity_permissions(&self.app_namespace, sub, &permissions)
                        .await?;
                    Ok(permissions)
                }
            }
        })
    }
}
