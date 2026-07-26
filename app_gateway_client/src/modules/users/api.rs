use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::ApiError;
use app_modules::base::users::models::{SetPasswordBody, User, UserCreate, UserUpdate};

pub async fn list_users(client: &ApiClient) -> Result<Vec<User>, ApiError> {
    let resp = client.request("GET", "/users", None).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn get_user(client: &ApiClient, user_id: &str) -> Result<User, ApiError> {
    let resp = client.request("GET", &format!("/users/{user_id}"), None).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn create_user(client: &ApiClient, body: &UserCreate) -> Result<User, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client.request("POST", "/users", Some(&json)).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn update_user(
    client: &ApiClient,
    user_id: &str,
    body: &UserUpdate,
) -> Result<User, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client
        .request("PUT", &format!("/users/{user_id}"), Some(&json))
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn delete_user(client: &ApiClient, user_id: &str) -> Result<(), ApiError> {
    client.request("DELETE", &format!("/users/{user_id}"), None).await?;
    Ok(())
}

pub async fn set_password(
    client: &ApiClient,
    user_id: &str,
    body: &SetPasswordBody,
) -> Result<User, ApiError> {
    let json = serde_json::to_string(body).map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = client
        .request("POST", &format!("/users/{user_id}/set_password"), Some(&json))
        .await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}
