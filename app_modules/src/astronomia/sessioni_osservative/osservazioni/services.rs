use std::sync::Arc;

use cornetti::{core::models::CornettiError, mongo::services::MongoDBService};
use validator::Validate;

use crate::astronomia::sessioni_osservative::{
    models::{Osservazione, OsservazioneInput},
    osservazioni::repos::OsservazioneRepository,
};

pub struct OsservazioneService {
    repository: OsservazioneRepository,
}

impl OsservazioneService {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self {
            repository: OsservazioneRepository::new(mongo),
        }
    }

    pub async fn list_osservazioni(
        &self,
        sessione_id: &str,
    ) -> Result<Vec<Osservazione>, CornettiError> {
        self.repository.list_by_session(sessione_id).await
    }

    pub async fn list_osservazioni_by_oggetto(
        &self,
        oggetto_id: &str,
    ) -> Result<Vec<Osservazione>, CornettiError> {
        self.repository.list_by_oggetto(oggetto_id).await
    }

    pub async fn get_osservazione(
        &self,
        sessione_id: &str,
        osservazione_id: &str,
    ) -> Result<Osservazione, CornettiError> {
        self.repository.get(sessione_id, osservazione_id).await
    }

    pub async fn create_osservazione(
        &self,
        sessione_id: &str,
        input: OsservazioneInput,
    ) -> Result<Osservazione, CornettiError> {
        input.validate()?;
        self.repository.create(sessione_id, input).await
    }

    pub async fn update_osservazione(
        &self,
        sessione_id: &str,
        osservazione_id: &str,
        input: OsservazioneInput,
    ) -> Result<Osservazione, CornettiError> {
        input.validate()?;
        self.repository.update(sessione_id, osservazione_id, input).await
    }

    pub async fn delete_osservazione(
        &self,
        sessione_id: &str,
        osservazione_id: &str,
    ) -> Result<(), CornettiError> {
        self.repository.delete(sessione_id, osservazione_id).await
    }
}
