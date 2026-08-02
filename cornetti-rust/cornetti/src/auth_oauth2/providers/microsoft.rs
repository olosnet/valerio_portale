use crate::auth_oauth2::models::OAuth2UserTransportData;
use crate::auth_oauth2::traits::OAuth2Provider;
use crate::core::models::CornettiResult;
use crate::errors;
use serde::{Deserialize, Serialize};

pub struct MicrosoftOAuth2Provider;

#[derive(Deserialize, Serialize)]
struct MicrosoftUserInfo {
    id: String,
    #[serde(rename = "userPrincipalName")]
    user_principal_name: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    mail: Option<String>,
}

impl OAuth2Provider for MicrosoftOAuth2Provider {
    fn name() -> &'static str {
        "microsoft"
    }

    fn auth_url() -> &'static str {
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
    }

    fn token_url() -> &'static str {
        "https://login.microsoftonline.com/common/oauth2/v2.0/token"
    }

    fn default_scopes() -> &'static [&'static str] {
        &["openid", "email", "profile", "User.Read"]
    }

    async fn get_user_info(
        http_client: &reqwest::Client,
        access_token: &str,
    ) -> CornettiResult<OAuth2UserTransportData> {
        let response = http_client
            .get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let user_info: MicrosoftUserInfo = response
            .json()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let raw = serde_json::to_value(&user_info).unwrap_or_default();

        let email = user_info.mail.or(user_info.user_principal_name);

        Ok(OAuth2UserTransportData {
            provider: Self::name().to_string(),
            provider_user_id: user_info.id,
            email,
            // Microsoft Graph does not expose an email verification flag:
            // `None` = unknown state, the decision is up to the consumer.
            email_verified: None,
            name: user_info.display_name,
            avatar_url: None,
            raw_data: raw,
        })
    }
}
