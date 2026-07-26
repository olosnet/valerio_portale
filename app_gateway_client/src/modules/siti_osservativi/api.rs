use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::ApiError;
use app_modules::astronomia::siti_osservativi::models::{
    SitoOsservativo, SitoOsservativoCreate, SitoOsservativoUpdate,
};

pub async fn list_siti(client: &ApiClient) -> Result<Vec<SitoOsservativo>, ApiError> {
    let resp = client.request("GET", "/siti_osservativi", None).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn get_sito(
    client: &ApiClient,
    sito_id: &str,
) -> Result<SitoOsservativo, ApiError> {
    let resp = client
        .request("GET", &format!("/siti_osservativi/{sito_id}"), None)
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn create_sito(
    client: &ApiClient,
    body: &SitoOsservativoCreate,
) -> Result<SitoOsservativo, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client
        .request("POST", "/siti_osservativi", Some(&json))
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn update_sito(
    client: &ApiClient,
    sito_id: &str,
    body: &SitoOsservativoUpdate,
) -> Result<SitoOsservativo, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client
        .request("PUT", &format!("/siti_osservativi/{sito_id}"), Some(&json))
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn delete_sito(client: &ApiClient, sito_id: &str) -> Result<(), ApiError> {
    client
        .request("DELETE", &format!("/siti_osservativi/{sito_id}"), None)
        .await?;
    Ok(())
}
