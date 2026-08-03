//! Modulo OAuth2: persistenza della metadata associata agli utenti e
//! implementazione dell'handler richiesto dal framework cornetti.

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

#[cfg(feature = "server")]
pub mod handler;
#[cfg(feature = "server")]
pub mod models;
#[cfg(feature = "server")]
pub mod repos;

#[cfg(feature = "server")]
use self::models::MongoOAuth2MetadataModel;

#[cfg(feature = "server")]
pub struct OAuth2Module;

#[cfg(feature = "server")]
impl BaseModule for OAuth2Module {
    fn module_name() -> &'static str {
        "oauth2"
    }

    fn module_version() -> i32 {
        1
    }

    fn module_permissions() -> &'static [&'static str] {
        &["oauth2"]
    }
}

#[cfg(feature = "server")]
impl MongoBaseModule for OAuth2Module {
    async fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = module_version;

        let collection_name = MongoOAuth2MetadataModel::collection_name();
        let collection = mongo.db().collection::<MongoOAuth2MetadataModel>(collection_name);
        let indexes = get_collection_indexes(collection.clone()).await;

        log::info!("Create {} indexes...", collection_name);

        if !indexes.contains(&"oauth2_provider_user_idx".to_string()) {
            let keys = doc! {
                "tenant_id": 1,
                "provider": 1,
                "provider_user_id": 1,
            };

            let options = IndexOptions::builder()
                .unique(true)
                .name(Some("oauth2_provider_user_idx".to_string()))
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
