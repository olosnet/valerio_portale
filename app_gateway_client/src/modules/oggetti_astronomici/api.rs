use serde::Deserialize;

use crate::modules::base::models::ApiError;
use crate::modules::base::{api_client::ApiClient, traits::CrudApi};
use app_modules::astronomia::oggetti_astronomici::models::{
    OggettoAstronomico, OggettoAstronomicoCreate, OggettoAstronomicoUpdate,
};

pub struct OggettiAstronomiciApi {
    api_client: Arc<ApiClient>,
}

impl BaseApi for OggettiAstronomiciApi {
    fn base_path(&self) -> &str {
        "/oggetti_astronomici"
    }

    fn api_client(&self) -> Arc<ApiClient> {
        self.api_client.clone()
    }
}

impl OggettiAstronomiciApi {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }

    pub async fn upload_oggetto_image(
        &self,
        id: &str,
        image_data: Vec<u8>,
        filename: &str,
        mime_type: &str,
    ) -> Result<OggettoAstronomico, ApiError> {
        let resp = self
            .api_client
            .upload_file(
                &format!("/{}/{}/image", self.base_path(), oggetto_id),
                image_data,
                filename,
                mime_type,
            )
            .await?;
        serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
    }
}

impl CrudApi<OggettoAstronomico, OggettoAstronomicoCreate, OggettoAstronomicoUpdate>
    for OggettiAstronomiciApi
{
}

impl PaginateApi<OggettoAstronomico> for OggettiAstronomiciApi {}
