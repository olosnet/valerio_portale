use crate::modules::base::{
    api_client::ApiClient,
    traits::{BaseApi, CrudApi},
};
use app_modules::astronomia::strumentazione::models::{
    Strumentazione, StrumentazioneCreate, StrumentazioneUpdate,
};
use std::sync::Arc;

pub struct StrumentazioneApi {
    api_client: Arc<ApiClient>,
}

impl StrumentazioneApi {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }
}

impl BaseApi for StrumentazioneApi {
    fn base_path(&self) -> &str {
        "/strumentazione"
    }

    fn api_client(&self) -> Arc<ApiClient> {
        Arc::clone(&self.api_client)
    }
}

impl CrudApi<Strumentazione, StrumentazioneCreate, StrumentazioneUpdate> for StrumentazioneApi {}
