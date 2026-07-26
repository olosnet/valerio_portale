use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::ApiError;
use app_modules::base::groups::models::{Group, GroupCreate, GroupUpdate};

pub async fn list_groups(client: &ApiClient) -> Result<Vec<Group>, ApiError> {
    let resp = client.request("GET", "/groups", None).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn get_group(client: &ApiClient, group_id: &str) -> Result<Group, ApiError> {
    let resp = client.request("GET", &format!("/groups/{group_id}"), None).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn create_group(client: &ApiClient, body: &GroupCreate) -> Result<Group, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client.request("POST", "/groups", Some(&json)).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn update_group(
    client: &ApiClient,
    group_id: &str,
    body: &GroupUpdate,
) -> Result<Group, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client
        .request("PUT", &format!("/groups/{group_id}"), Some(&json))
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn delete_group(client: &ApiClient, group_id: &str) -> Result<(), ApiError> {
    client.request("DELETE", &format!("/groups/{group_id}"), None).await?;
    Ok(())
}
