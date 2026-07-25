use std::sync::Arc;

use cornetti::{
    core::models::CornettiResult,
    errors,
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

    pub async fn list_users(&self) -> CornettiResult<Vec<User>> {
        self.repository.list().await
    }

    pub async fn get_user(&self, user_id: &str) -> CornettiResult<User> {
        self.repository.get(user_id).await
    }

    pub async fn create_user(&self, user_create: UserCreate) -> CornettiResult<User> {
        user_create.validate()?;
        self.repository.create(user_create).await
    }

    pub async fn update_user(
        &self,
        user_id: &str,
        user_update: UserUpdate,
    ) -> CornettiResult<User> {
        user_update.validate()?;

        let user_updated = self.repository.update(user_id, &user_update).await?;
        if let Some(ref email) = user_updated.email {
            self.cache_repository
                .remove_identity_permissions(self.app_namespace, email)
                .await?;
        }

        Ok(user_updated)
    }

    pub async fn delete_user(&self, user_id: &str) -> CornettiResult<()> {
        let existing = self.repository.get(user_id).await?;
        if existing.default {
            return Err(errors::not_allowed::resource_deletion_not_allowed());
        }
        let result = self.repository.delete(user_id, false).await?;
        Ok(result)
    }

    pub async fn set_password(
        &self,
        user_id: &str,
        set_password_body: SetPasswordBody,
    ) -> CornettiResult<User> {
        set_password_body.validate()?;
        self.repository
            .set_password(user_id, &set_password_body.password)
            .await
    }
}
