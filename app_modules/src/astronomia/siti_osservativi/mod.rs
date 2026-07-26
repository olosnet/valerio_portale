#[cfg(feature = "server")]
use cornetti::{
    core::traits::BaseModule,
    mongo::{services::MongoDBService, traits::MongoBaseModule},
};

pub mod models;
#[cfg(feature = "server")]
pub mod repos;
#[cfg(feature = "server")]
pub mod services;

#[cfg(feature = "server")]
pub struct SitiOsservativiModule;

#[cfg(feature = "server")]
impl BaseModule for SitiOsservativiModule {
    fn module_name() -> &'static str {
        "siti_osservativi"
    }

    fn module_version() -> i32 {
        1
    }

    fn module_permissions() -> &'static [&'static str] {
        &["all", "astronomia"]
    }
}

#[cfg(feature = "server")]
impl MongoBaseModule for SitiOsservativiModule {
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
