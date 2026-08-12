use crate::modules::base::{
    api_client::ApiClient,
    models::{ApiError, ApiHttpMethod},
    traits::BaseApi,
};
use app_modules::statics::models::StaticsResponse;
use std::sync::Arc;

pub struct StaticsApi {
    api_client: Arc<ApiClient>,
}

impl BaseApi for StaticsApi {
    fn base_path(&self) -> &str {
        "/statics"
    }

    fn api_client(&self) -> Arc<ApiClient> {
        self.api_client.clone()
    }
}

impl StaticsApi {
    pub async fn list(&self) -> Result<StaticsResponse, ApiError> {
        let resp = self
            .api_client
            .request(&ApiHttpMethod::GET, self.base_path(), None)
            .await?;
        serde_json::from_str(&resp).map_err(|e| ApiError::DeserializationFailed(e.to_string()))
    }
}
