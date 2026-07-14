use std::sync::Arc;

use cornetti::{
    auth::{models::AuthorizationPermission, traits::IdentityAuthorization},
    core::models::CornettiError,
    mongo::services::MongoDBService,
    redis::services::RedisDBService,
};
use validator::Validate;

use crate::base::users::{
    models::{SetPasswordBody, User, UserCreate, UserUpdate},
    repos::{UsersCacheRepository, UsersRepository},
};

pub struct UsersService<'a> {
    pub repository: UsersRepository,
    pub cache_repository: UsersCacheRepository,
    pub app_namespace: &'a String,
}

impl<'a> UsersService<'a> {
    pub fn new(
        mongo: Arc<MongoDBService>,
        redis: Arc<RedisDBService>,
        app_namespace: &'a String,
    ) -> Self {
        UsersService {
            repository: UsersRepository::new(mongo),
            cache_repository: UsersCacheRepository::new(redis),
            app_namespace,
        }
    }

    pub async fn list_users(&self) -> Result<Vec<User>, CornettiError> {
        self.repository.list().await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<User, CornettiError> {
        self.repository.get(user_id).await
    }

    pub async fn create_user(&self, user_create: UserCreate) -> Result<User, CornettiError> {
        user_create.validate()?;
        self.repository.create(user_create).await
    }

    pub async fn update_user(
        &self,
        user_id: &str,
        user_update: UserUpdate,
    ) -> Result<User, CornettiError> {
        user_update.validate()?;
        let user_updated = self.repository.update(user_id, user_update).await?;
        if let Some(ref email) = user_updated.email {
            self.cache_repository
                .remove_identity_permissions(self.app_namespace, email)
                .await?;
        }

        Ok(user_updated)
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<(), CornettiError> {
        let result = self.repository.delete(user_id, false).await?;
        Ok(result)
    }

    pub async fn set_password(
        &self,
        user_id: &str,
        set_password_body: SetPasswordBody,
    ) -> Result<User, CornettiError> {
        set_password_body.validate()?;
        self.repository
            .set_password(user_id, &set_password_body.password)
            .await
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
        Output = Result<std::collections::HashMap<String, AuthorizationPermission>, CornettiError>,
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
