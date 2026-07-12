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

pub struct FileManagerImagesModule;

impl BaseModule for FileManagerImagesModule {
    fn module_name() -> &'static str {
        "filemanager_images"
    }

    fn module_version() -> i32 {
        1
    }

    fn module_permissions() -> &'static [&'static str] {
        &["all", "filemanager_images"]
    }
}

impl MongoBaseModule for FileManagerImagesModule {
    async fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = module_version;

        let collection_name = repos::MongoImageFileManagerResize::collection_name();
        let collection = mongo
            .db()
            .collection::<repos::MongoImageFileManagerResize>(collection_name);
        let indexes = get_collection_indexes(collection.clone()).await;

        log::info!("Create {} indexes...", collection_name);

        if !indexes.contains(&"default_idx".to_string()) {

            let keys = doc! { "parent_filename": 1, "resize_slug": 1 };

            let options = IndexOptions::builder()
                .unique(true)
                .name(Some("default_idx".to_string()))
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
