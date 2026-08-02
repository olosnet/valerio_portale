use crate::auth_oauth2::models::OAuth2UserTransportData;
use crate::auth_oauth2::traits::OAuth2Provider;
use crate::core::models::CornettiResult;
use crate::errors;
use serde::{Deserialize, Serialize};

pub struct FacebookOAuth2Provider;

#[derive(Deserialize, Serialize)]
struct FacebookPictureData {
    url: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct FacebookPicture {
    data: Option<FacebookPictureData>,
}

#[derive(Deserialize, Serialize)]
struct FacebookUserInfo {
    id: String,
    name: Option<String>,
    email: Option<String>,
    picture: Option<FacebookPicture>,
}

impl OAuth2Provider for FacebookOAuth2Provider {
    fn name() -> &'static str {
        "facebook"
    }

    fn auth_url() -> &'static str {
        "https://www.facebook.com/v22.0/dialog/oauth"
    }

    fn token_url() -> &'static str {
        "https://graph.facebook.com/v22.0/oauth/access_token"
    }

    fn default_scopes() -> &'static [&'static str] {
        &["email", "public_profile"]
    }

    async fn get_user_info(
        http_client: &reqwest::Client,
        access_token: &str,
    ) -> CornettiResult<OAuth2UserTransportData> {
        // The access token goes in the Authorization header, not in the query string:
        // reqwest includes the full URL in the Display of errors, which ends up
        // in the logs through `internal_detail`.
        let response = http_client
            .get("https://graph.facebook.com/me")
            .query(&[("fields", "id,name,email,picture")])
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let user_info: FacebookUserInfo = response
            .json()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let raw = serde_json::to_value(&user_info).unwrap_or_default();

        Ok(OAuth2UserTransportData {
            provider: Self::name().to_string(),
            provider_user_id: user_info.id,
            email: user_info.email,
            // The /me endpoint does not expose an email verification flag:
            // `None` = unknown state, the decision is up to the consumer.
            email_verified: None,
            name: user_info.name,
            avatar_url: user_info.picture.and_then(|p| p.data).and_then(|d| d.url),
            raw_data: raw,
        })
    }
}
