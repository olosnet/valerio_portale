use std::sync::Arc;

use crate::modules::base::traits::CrudApi;
use crate::modules::base::{api_client::ApiClient, traits::BaseApi};
use app_modules::astronomia::siti_osservativi::models::{
    SitoOsservativo, SitoOsservativoCreate, SitoOsservativoUpdate,
};

pub struct SitiOsservativiApi {
    api_client: Arc<ApiClient>,
}

impl SitiOsservativiApi {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }
}

impl BaseApi for SitiOsservativiApi {
    fn base_path(&self) -> &str {
        "/siti_osservativi"
    }

    fn api_client(&self) -> Arc<ApiClient> {
        self.api_client.clone()
    }
}

impl CrudApi<SitoOsservativo, SitoOsservativoCreate, SitoOsservativoUpdate> for SitiOsservativiApi {}
