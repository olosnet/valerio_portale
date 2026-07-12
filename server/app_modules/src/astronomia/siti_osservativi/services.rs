use std::sync::Arc;

use cornetti::{core::models::CornettiError, mongo::services::MongoDBService};
use validator::Validate;

use crate::astronomia::siti_osservativi::{
    models::{SitoOsservativo, SitoOsservativoCreate, SitoOsservativoUpdate},
    repos::SitiOsservativiRepository,
};

pub struct SitiOsservativiService {
    repository: SitiOsservativiRepository,
}

impl SitiOsservativiService {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self {
            repository: SitiOsservativiRepository::new(mongo),
        }
    }

    pub async fn list_siti_osservativi(&self) -> Result<Vec<SitoOsservativo>, CornettiError> {
        self.repository.list().await
    }

    pub async fn get_sito_osservativo(
        &self,
        sito_id: &str,
    ) -> Result<SitoOsservativo, CornettiError> {
        self.repository.get(sito_id).await
    }

    pub async fn create_sito_osservativo(
        &self,
        sito_create: SitoOsservativoCreate,
    ) -> Result<SitoOsservativo, CornettiError> {
        sito_create.validate()?;
        self.repository.create(sito_create).await
    }

    pub async fn update_sito_osservativo(
        &self,
        sito_id: &str,
        sito_update: SitoOsservativoUpdate,
    ) -> Result<SitoOsservativo, CornettiError> {
        sito_update.validate()?;
        self.repository.update(sito_id, sito_update).await
    }

    pub async fn delete_sito_osservativo(&self, sito_id: &str) -> Result<(), CornettiError> {
        self.repository.delete(sito_id).await
    }
}
