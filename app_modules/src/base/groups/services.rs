use cornetti::{
    core::models::CornettiResult,
    errors,
    mongo::services::MongoDBService, redis::services::RedisDBService,
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    base::groups::{
        models::{Group, GroupCreate, GroupUpdate},
        repos::GroupsRepository,
    },
    base::users::repos::{UsersCacheRepository, UsersRepository},
};

pub struct GroupService<'a> {
    repository: GroupsRepository,
    users_repository: UsersRepository,
    users_cache_repository: UsersCacheRepository,
    app_namespace: &'a str,
}

impl<'a> GroupService<'a> {
    pub fn new(
        mongo: Arc<MongoDBService>,
        redis: Arc<RedisDBService>,
        app_namespace: &'a str,
    ) -> Self {
        Self {
            repository: GroupsRepository::new(mongo.clone()),
            users_repository: UsersRepository::new(mongo),
            users_cache_repository: UsersCacheRepository::new(redis),
            app_namespace,
        }
    }

    pub async fn list_groups(&self) -> CornettiResult<Vec<Group>> {
        self.repository.list().await
    }

    pub async fn get_group(&self, group_id: &str) -> CornettiResult<Group> {
        self.repository.get(group_id).await
    }

    pub async fn create_group(&self, group_create: GroupCreate) -> CornettiResult<Group> {
        group_create.validate()?;
        self.repository.create(group_create).await
    }

    pub async fn update_group(
        &self,
        group_id: &str,
        group_update: GroupUpdate,
    ) -> CornettiResult<Group> {
        group_update.validate()?;

        let result = self.repository.update(group_id, &group_update).await;

        match result {
            Ok(group) => {
                // Clear cached permissions for users in the group
                let users = self
                    .users_repository
                    .get_users_from_group_id(group_id)
                    .await?;
                for user in users {
                    self.users_cache_repository
                        .remove_identity_permissions(self.app_namespace, &user.email.unwrap())
                        .await?;
                }
                Ok(group)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn delete_group(&self, group_id: &str) -> CornettiResult<()> {
        let existing = self.repository.get(group_id).await?;
        if existing.default {
            return Err(errors::not_allowed::resource_deletion_not_allowed());
        }

        let result = self.repository.delete(group_id).await;
        match result {
            Ok(_) => {
                // Clear cached permissions for users in the group
                let users = self
                    .users_repository
                    .get_users_from_group_id(group_id)
                    .await?;
                for user in users {
                    self.users_cache_repository
                        .remove_identity_permissions(
                            &self.app_namespace.to_string(),
                            &user.email.unwrap(),
                        )
                        .await?;
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
