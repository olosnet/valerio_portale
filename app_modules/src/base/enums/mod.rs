#[cfg(feature = "server")]
use bson::doc;
#[cfg(feature = "server")]
use cornetti::{
    core::traits::BaseModule,
    mongo::{
        helpers::get_collection_indexes,
        services::MongoDBService,
        traits::{MongoBaseModel, MongoBaseModule},
    },
};
#[cfg(feature = "server")]
use mongodb::{IndexModel, options::IndexOptions};

pub mod models;
#[cfg(feature = "server")]
pub mod repos;
#[cfg(feature = "server")]
pub mod services;

#[cfg(feature = "server")]
use self::repos::MongoEnumModel;

#[cfg(feature = "server")]
pub struct EnumsModule;

#[cfg(feature = "server")]
impl BaseModule for EnumsModule {
    fn module_name() -> &'static str {
        "enums"
    }

    fn module_version() -> i32 {
        1
    }

    fn module_permissions() -> &'static [&'static str] {
        &["all", "configs"]
    }
}

#[cfg(feature = "server")]
impl MongoBaseModule for EnumsModule {
    async fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = module_version;

        let collection_name = MongoEnumModel::collection_name();
        let collection = mongo.db().collection::<MongoEnumModel>(collection_name);
        let indexes = get_collection_indexes(collection.clone()).await;

        log::info!("Create {} indexes...", collection_name);

        if !indexes.contains(&"category_idx".to_string()) {
            let keys = doc! { "category": 1 };
            let options = IndexOptions::builder()
                .name(Some("category_idx".to_string()))
                .build();
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
    }
}
