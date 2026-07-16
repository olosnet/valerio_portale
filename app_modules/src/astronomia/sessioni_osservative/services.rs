use std::sync::Arc;

use cornetti::{core::models::CornettiError, mongo::services::MongoDBService};
use validator::Validate;

use crate::astronomia::sessioni_osservative::{
    models::{
        SessioneOsservativa, SessioneOsservativaCreate, SessioneOsservativaUpdate,
    },
    repos::SessioniOsservativeRepository,
};

pub struct SessioniOsservativeService {
    repository: SessioniOsservativeRepository,
}

impl SessioniOsservativeService {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self {
            repository: SessioniOsservativeRepository::new(mongo),
        }
    }

    pub async fn list_sessioni_osservative(
        &self,
    ) -> Result<Vec<SessioneOsservativa>, CornettiError> {
        self.repository.list().await
    }

    pub async fn get_sessione_osservativa(
        &self,
        sessione_id: &str,
    ) -> Result<SessioneOsservativa, CornettiError> {
        self.repository.get(sessione_id).await
    }

    pub async fn create_sessione_osservativa(
        &self,
        sessione_create: SessioneOsservativaCreate,
    ) -> Result<SessioneOsservativa, CornettiError> {
        sessione_create.validate()?;
        self.repository.create(sessione_create).await
    }

    pub async fn update_sessione_osservativa(
        &self,
        sessione_id: &str,
        sessione_update: SessioneOsservativaUpdate,
    ) -> Result<SessioneOsservativa, CornettiError> {
        sessione_update.validate()?;
        self.repository.update(sessione_id, sessione_update).await
    }

    pub async fn delete_sessione_osservativa(
        &self,
        sessione_id: &str,
    ) -> Result<(), CornettiError> {
        self.repository.delete(sessione_id).await
    }
}
