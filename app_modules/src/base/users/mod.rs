use bson::doc;
use cornetti::{
    core::traits::BaseModule,
    mongo::{
        helpers::get_collection_indexes,
        services::MongoDBService,
        traits::{MongoBaseModel, MongoBaseModule},
    },
};
use mongodb::{IndexModel, options::IndexOptions};
pub mod models;
pub mod repos;
pub mod services;

use self::repos::MongoUserModel;

pub struct UsersModule;

impl BaseModule for UsersModule {
    fn module_name() -> &'static str {
        "users"
    }

    fn module_version() -> i32 {
        1
    }

    fn module_permissions() -> &'static [&'static str] {
        &["all", "users"]
    }
}

impl MongoBaseModule for UsersModule {
    async fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = module_version;
        // No indexes for now

        let collection_name = MongoUserModel::collection_name();
        let collection = mongo.db().collection::<MongoUserModel>(collection_name);
        let indexes = get_collection_indexes(collection.clone()).await;

        log::info!("Create {} indexes...", collection_name);

        if !indexes.contains(&"email_idx".to_string()) {

            let keys = doc! { "email": 1 };

            let options = IndexOptions::builder()
                .unique(true)
                .name(Some("email_idx".to_string()))
                .build();

            // Crea l'indice
            let index = IndexModel::builder()
                .keys(keys)
                .options(Some(options))
                .build();

            collection.create_index(index).await?;
        }
        Ok(())
    }

    async fn seed(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = mongo;
        let _ = module_version;
        Ok(())
        // No seed for now
    }
}
