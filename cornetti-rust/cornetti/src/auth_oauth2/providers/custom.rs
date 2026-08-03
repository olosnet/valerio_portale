//! Generic OIDC provider configured at runtime (custom providers).
//!
//! Unlike built-in providers (whose URLs are hardcoded in the
//! [`OAuth2Provider`](crate::auth_oauth2::traits::OAuth2Provider) trait),
//! custom providers get their endpoints from the `[auth.oauth2.providers]`
//! configuration (`auth_url`, `token_url`, `userinfo_url`). The user info is
//! fetched from the standard OIDC `/userinfo` endpoint and parsed with the
//! standard claims (`sub`, `email`, `email_verified`, `name`, `picture`) —
//! Kanidm, Keycloak, Authelia, Authentik, etc. all expose this shape.

use crate::auth_oauth2::confs::OAuth2ProviderConf;
use crate::auth_oauth2::models::OAuth2UserTransportData;
use crate::core::models::CornettiResult;
use crate::errors;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct OidcUserInfo {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

/// Fetches the authenticated user's data from a custom provider's
/// `/userinfo` endpoint.
///
/// # Errors
/// Returns `user_info_error` if the `userinfo_url` is not configured or the
/// API call fails.
pub async fn get_user_info(
    http_client: &reqwest::Client,
    access_token: &str,
    provider_conf: &OAuth2ProviderConf,
) -> CornettiResult<OAuth2UserTransportData> {
    let userinfo_url = provider_conf.userinfo_url.clone().ok_or_else(|| {
        errors::auth_oauth2_errors::user_info_error().with_internal_detail(format!(
            "Custom provider '{}' has no userinfo_url configured",
            provider_conf.name
        ))
    })?;

    let response = http_client
        .get(&userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            errors::auth_oauth2_errors::user_info_error()
                .with_internal_detail(e.to_string())
        })?;

    let user_info: OidcUserInfo = response.json().await.map_err(|e| {
        errors::auth_oauth2_errors::user_info_error().with_internal_detail(e.to_string())
    })?;

    let raw = serde_json::to_value(&user_info).unwrap_or_default();

    Ok(OAuth2UserTransportData {
        provider: provider_conf.name.clone(),
        provider_user_id: user_info.sub,
        email: user_info.email,
        email_verified: user_info.email_verified,
        name: user_info.name,
        avatar_url: user_info.picture,
        raw_data: raw,
    })
}
