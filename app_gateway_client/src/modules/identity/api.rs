use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::ApiError;
use crate::modules::identity::models::{UserIdentity, UserIdentityUpdate, UserIdentityUpdatePassword};

pub async fn get_identity(client: &ApiClient) -> Result<UserIdentity, ApiError> {
    let resp = client.request("GET", "/identity", None).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn update_profile(
    client: &ApiClient,
    body: &UserIdentityUpdate,
) -> Result<UserIdentity, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client.request("PUT", "/identity", Some(&json)).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn update_password(
    client: &ApiClient,
    body: &UserIdentityUpdatePassword,
) -> Result<(), ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    client.request("POST", "/identity/password", Some(&json)).await?;
    Ok(())
}
