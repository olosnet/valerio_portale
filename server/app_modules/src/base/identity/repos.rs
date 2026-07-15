use std::sync::Arc;

use bson::doc;
use cornetti::{
    core::{errors, helpers::sec::hash_password, models::CornettiError, traits::BaseModule},
    mongo::services::MongoDBService,
};
use mongodb::{Collection, options::ReturnDocument};

use crate::base::users::{
    UsersModule,
    models::{User, UserIdentity},
    repos::MongoUserModel,
};

use super::models::UserIdentityUpdate;

pub struct IdentityRepository {
    mongo: Arc<MongoDBService>,
}

impl IdentityRepository {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self { mongo }
    }

    pub async fn get_identity(&self, email: &str) -> Result<UserIdentity, CornettiError> {
        let users_repo = crate::base::users::repos::UsersRepository::new(self.mongo.clone());
        users_repo.get_identity(&email.to_string()).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, CornettiError> {
        let collection_name = UsersModule::module_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        match collection.find_one(doc! { "email": email }).await? {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn get_user_password_hash(&self, email: &str) -> Result<String, CornettiError> {
        let collection_name = UsersModule::module_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        match collection.find_one(doc! { "email": email }).await? {
            Some(item) => item.password.ok_or_else(|| {
                errors::internal_server_error::generic_error("User has no password set".to_string())
            }),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn update_profile(
        &self,
        email: &str,
        dto: &UserIdentityUpdate,
    ) -> Result<User, CornettiError> {
        let collection_name = UsersModule::module_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let current = self.get_user_by_email(email).await?;

        let name = dto.name.as_deref().unwrap_or(current.name.as_deref().unwrap_or(""));
        let surname = dto.surname.as_deref().unwrap_or(current.surname.as_deref().unwrap_or(""));

        let modified = chrono::Utc::now();

        match collection
            .find_one_and_update(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "name": name,
                        "surname": surname,
                        "modified": bson::DateTime::from_chrono(modified),
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn set_profile_image(&self, email: &str, filename: &str) -> Result<User, CornettiError> {
        let collection_name = UsersModule::module_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let modified = chrono::Utc::now();

        match collection
            .find_one_and_update(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "profile_image": filename,
                        "modified": bson::DateTime::from_chrono(modified),
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }

    pub async fn update_password(&self, email: &str, new_password: &str) -> Result<User, CornettiError> {
        let collection_name = UsersModule::module_name();
        let collection: Collection<MongoUserModel> = self.mongo.db().collection(collection_name);

        let hashed = hash_password(&new_password.to_string());
        let modified = chrono::Utc::now();

        match collection
            .find_one_and_update(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "password": hashed,
                        "modified": bson::DateTime::from_chrono(modified),
                    }
                },
            )
            .return_document(ReturnDocument::After)
            .await?
        {
            Some(item) => Ok(item.into()),
            None => Err(errors::not_found::item_not_found()),
        }
    }
}
