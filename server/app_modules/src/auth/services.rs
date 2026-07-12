use std::sync::Arc;

use crate::users::repos::UsersRepository;
use cornetti::{
    actix::auth::helpers::invalidate_session,
    auth::{confs::JwtAuthConf, models::JwtDefaultClaims, traits::SessionStore},
    core::errors,
    mongo::services::MongoDBService,
};

pub struct AuthenticationService<'a, S: SessionStore> {
    users_repository: UsersRepository,
    conf: &'a JwtAuthConf,
    session_store: Option<Arc<S>>,
    tenant_id: &'a str,
}

impl<'a, S: SessionStore> AuthenticationService<'a, S> {
    pub fn new(
        mongo: Arc<MongoDBService>,
        conf: &'a JwtAuthConf,
        session_store: Option<Arc<S>>,
        tenant_id: &'a str,
    ) -> AuthenticationService<'a, S> {
        AuthenticationService {
            users_repository: UsersRepository::new(mongo),
            conf,
            session_store,
            tenant_id,
        }
    }

    pub async fn login(
        &self,
        login: cornetti::auth::models::DefaultLoginBody,
        req: actix_web::HttpRequest,
    ) -> Result<
        (
            cornetti::auth::models::DefaultLoginResponse<crate::users::models::User>,
            Option<actix_web::cookie::Cookie<'_>>,
            Option<actix_web::cookie::Cookie<'_>>,
            Option<actix_web::cookie::Cookie<'_>>,
            Option<actix_web::cookie::Cookie<'_>>,
        ),
        cornetti::core::models::CornettiError,
    > {
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

    pub async fn logout(
        &self,
        claims: Option<JwtDefaultClaims>,
    ) -> Result<Vec<&str>, cornetti::core::models::CornettiError> {
        if let Some(c) = claims {
            let _ = self.users_repository.get_identity(&c.sub).await?;

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

    pub async fn identity(
        &self,
        claims: Option<JwtDefaultClaims>,
    ) -> Result<crate::users::models::UserIdentity, cornetti::core::models::CornettiError> {
        if let Some(c) = claims {
            return self.users_repository.get_identity(&c.sub).await;
        }

        Err(errors::not_found::item_not_found())
    }

    pub async fn refresh(
        &self,
        claims: Option<JwtDefaultClaims>,
        req: actix_web::HttpRequest,
    ) -> Result<
        (
            cornetti::auth::models::RefreshAuthResponseDto<crate::users::models::UserIdentity>,
            Option<actix_web::cookie::Cookie<'_>>,
            Option<actix_web::cookie::Cookie<'_>>,
        ),
        cornetti::core::models::CornettiError,
    > {
        if let Some(c) = claims {
            let user = self.users_repository.get_identity(&c.sub).await?;
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
