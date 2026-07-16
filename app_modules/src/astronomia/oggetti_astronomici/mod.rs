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

pub mod helpers;
pub mod models;
pub mod repos;
pub mod services;

use self::repos::MongoOggettoAstronomicoModel;

pub struct OggettiAstronomiciModule;

impl BaseModule for OggettiAstronomiciModule {
    fn module_name() -> &'static str {
        "oggetti_astronomici"
    }

    fn module_version() -> i32 {
        2
    }

    fn module_permissions() -> &'static [&'static str] {
        &["all", "astronomia"]
    }
}

impl MongoBaseModule for OggettiAstronomiciModule {
    async fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = module_version;

        let collection_name = MongoOggettoAstronomicoModel::collection_name();
        let collection = mongo
            .db()
            .collection::<MongoOggettoAstronomicoModel>(collection_name);
        let indexes = get_collection_indexes(collection.clone()).await;

        log::info!("Create {} indexes...", collection_name);

        // Indice per ricerca su cataloghi.extended (regex prefix)
        if !indexes.contains(&"cataloghi_extended_idx".to_string()) {
            let keys = doc! { "cataloghi.extended": 1 };
            let options = IndexOptions::builder()
                .name(Some("cataloghi_extended_idx".to_string()))
                .build();
            let index = IndexModel::builder()
                .keys(keys)
                .options(Some(options))
                .build();

            collection.create_index(index).await?;
        }

        // Indice per ricerca diretta su cataloghi.catalog_nr
        if !indexes.contains(&"cataloghi_catalog_nr_idx".to_string()) {
            let keys = doc! { "cataloghi.catalog_nr": 1 };
            let options = IndexOptions::builder()
                .name(Some("cataloghi_catalog_nr_idx".to_string()))
                .build();
            let index = IndexModel::builder()
                .keys(keys)
                .options(Some(options))
                .build();

            collection.create_index(index).await?;
        }

        // Indice per ricerca su nome_comune
        if !indexes.contains(&"nome_comune_idx".to_string()) {
            let keys = doc! { "nome_comune": 1 };
            let options = IndexOptions::builder()
                .name(Some("nome_comune_idx".to_string()))
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
