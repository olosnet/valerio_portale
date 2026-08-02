use crate::auth_oauth2::models::OAuth2UserTransportData;
use crate::auth_oauth2::traits::OAuth2Provider;
use crate::core::models::CornettiResult;
use crate::errors;
use serde::{Deserialize, Serialize};

pub struct GoogleOAuth2Provider;

#[derive(Deserialize, Serialize)]
struct GoogleUserInfo {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

impl OAuth2Provider for GoogleOAuth2Provider {
    fn name() -> &'static str {
        "google"
    }

    fn auth_url() -> &'static str {
        "https://accounts.google.com/o/oauth2/v2/auth"
    }

    fn token_url() -> &'static str {
        "https://oauth2.googleapis.com/token"
    }

    fn default_scopes() -> &'static [&'static str] {
        &["openid", "email", "profile"]
    }

    async fn get_user_info(
        http_client: &reqwest::Client,
        access_token: &str,
    ) -> CornettiResult<OAuth2UserTransportData> {
        let response = http_client
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let user_info: GoogleUserInfo = response
            .json()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let raw = serde_json::to_value(&user_info).unwrap_or_default();

        Ok(OAuth2UserTransportData {
            provider: Self::name().to_string(),
            provider_user_id: user_info.sub,
            email: user_info.email,
            email_verified: user_info.email_verified,
            name: user_info.name,
            avatar_url: user_info.picture,
            raw_data: raw,
        })
    }
}
