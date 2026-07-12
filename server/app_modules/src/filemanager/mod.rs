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

pub mod repos;
pub mod services;

pub struct FileManagerModule;

impl BaseModule for FileManagerModule {
    fn module_name() -> &'static str {
        "filemanager"
    }

    fn module_version() -> i32 {
        1
    }

    fn module_permissions() -> &'static [&'static str] {
        &[]
    }
}

impl MongoBaseModule for FileManagerModule {
    async fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = module_version;
        // No indexes for now

        let collection_name = repos::MongoFileManagerModel::collection_name();
        let collection = mongo
            .db()
            .collection::<repos::MongoFileManagerModel>(collection_name);
        let indexes = get_collection_indexes(collection.clone()).await;
        
        log::info!("Create {} indexes...", collection_name);

        if !indexes.contains(&"filename_idx".to_string()) {

            let keys = doc! { "filename": 1, "app_source" : 1 };

            let options = IndexOptions::builder()
                .unique(false)
                .name(Some("filename_idx".to_string()))
                .build();

            // Create the index
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
