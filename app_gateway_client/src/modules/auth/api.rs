use crate::modules::auth::models::{DefaultLoginResponse, OAuth2ProvidersResponse};
use crate::modules::base::api_client::ApiClient;
use crate::modules::base::models::ApiError;

pub async fn login(
    client: &ApiClient,
    username: &str,
    password: &str,
) -> Result<DefaultLoginResponse, ApiError> {
    let body = serde_json::json!({ "username": username, "password": password });
    let resp = client.request("POST", "/auth/login", Some(&body.to_string())).await?;
    serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))
}

pub async fn logout(client: &ApiClient) -> Result<(), ApiError> {
    client.request("POST", "/auth/logout", None).await?;
    Ok(())
}

pub async fn refresh(
    client: &ApiClient,
) -> Result<app_modules::base::identity::models::UserIdentity, ApiError> {
    let resp = client.request("POST", "/auth/refresh", None).await?;
    let dto: crate::modules::auth::models::RefreshAuthResponse =
        serde_json::from_str(&resp).map_err(|e| ApiError::Network(e.to_string()))?;
    Ok(dto.identity)
}

/// Provider OAuth2 esposti dal server. Endpoint presente solo quando OAuth2 è
/// abilitato: con 404 (disabilitato) ritorna `None`.
pub async fn oauth2_providers(
    client: &ApiClient,
) -> Result<Option<OAuth2ProvidersResponse>, ApiError> {
    match client.request("GET", "/auth/oauth2/providers", None).await {
        Ok(resp) => serde_json::from_str(&resp)
            .map(Some)
            .map_err(|e| ApiError::Network(e.to_string())),
        Err(ApiError::Http(404, _)) => Ok(None),
        Err(e) => Err(e),
    }
}
