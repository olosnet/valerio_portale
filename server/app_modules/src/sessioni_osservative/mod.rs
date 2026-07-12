use cornetti::{
    core::traits::BaseModule,
    mongo::{services::MongoDBService, traits::MongoBaseModule},
};

pub mod models;
pub mod osservazioni;
pub mod repos;
pub mod services;

pub struct SessioniOsservativeModule;

impl BaseModule for SessioniOsservativeModule {
    fn module_name() -> &'static str {
        "sessioni_osservative"
    }

    fn module_version() -> i32 {
        1
    }

    fn module_permissions() -> &'static [&'static str] {
        &["all", "astronomia"]
    }
}

impl MongoBaseModule for SessioniOsservativeModule {
    async fn create_indexes(
        mongo: &MongoDBService,
        module_version: i32,
    ) -> Result<(), mongodb::error::Error> {
        let _ = mongo;
        let _ = module_version;
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
