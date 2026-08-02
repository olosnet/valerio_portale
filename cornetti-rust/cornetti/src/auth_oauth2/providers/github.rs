use crate::auth_oauth2::models::OAuth2UserTransportData;
use crate::auth_oauth2::traits::OAuth2Provider;
use crate::core::models::CornettiResult;
use crate::errors;
use serde::{Deserialize, Serialize};

pub struct GitHubOAuth2Provider;

#[derive(Deserialize, Serialize)]
struct GitHubUserInfo {
    id: i64,
    login: Option<String>,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

// GitHub also returns verified emails from a separate endpoint.
// Here we use the /user endpoint, which requires the "user:email" scope for email.
#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

impl OAuth2Provider for GitHubOAuth2Provider {
    fn name() -> &'static str {
        "github"
    }

    fn auth_url() -> &'static str {
        "https://github.com/login/oauth/authorize"
    }

    fn token_url() -> &'static str {
        "https://github.com/login/oauth/access_token"
    }

    fn default_scopes() -> &'static [&'static str] {
        &["user:email", "read:user"]
    }

    async fn get_user_info(
        http_client: &reqwest::Client,
        access_token: &str,
    ) -> CornettiResult<OAuth2UserTransportData> {
        // Fetch user profile
        let response = http_client
            .get("https://api.github.com/user")
            .header("User-Agent", "cornetti-oauth2")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let user_info: GitHubUserInfo = response
            .json()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::user_info_error()
                    .with_internal_detail(e.to_string())
            })?;

        let raw = serde_json::to_value(&user_info).unwrap_or_default();

        // Retrieve the primary verified email from the dedicated endpoint.
        // If the call fails, fall back to the public profile email, which GitHub
        // does NOT guarantee as verified: in that case `email_verified` stays
        // `None` (unknown state) instead of falsely claiming `true`.
        let (email, email_verified) = match http_client
            .get("https://api.github.com/user/emails")
            .header("User-Agent", "cornetti-oauth2")
            .bearer_auth(access_token)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<Vec<GitHubEmail>>().await {
                Ok(emails) => match emails.into_iter().find(|e| e.primary && e.verified) {
                    Some(e) => (Some(e.email), Some(true)),
                    None => (user_info.email, None),
                },
                Err(_) => (user_info.email, None),
            },
            Err(_) => (user_info.email, None),
        };

        Ok(OAuth2UserTransportData {
            provider: Self::name().to_string(),
            provider_user_id: user_info.id.to_string(),
            email,
            email_verified,
            name: user_info.name.or(user_info.login),
            avatar_url: user_info.avatar_url,
            raw_data: raw,
        })
    }
}
