use crate::auth_oauth2::models::OAuth2UserTransportData;
use crate::auth_oauth2::traits::OAuth2Provider;
use crate::core::models::CornettiResult;
use crate::errors;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AppleOAuth2Provider;

/// Expected issuer in Apple id_tokens.
const APPLE_ISSUER: &str = "https://appleid.apple.com";

#[derive(Deserialize, Serialize)]
struct AppleIdTokenClaims {
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<i64>,
    sub: String,
    email: Option<String>,
    /// Apple sends this as either a boolean or a string "true"/"false" depending
    /// on the API version: both forms are accepted.
    email_verified: Option<serde_json::Value>,
    name: Option<String>,
}

/// Client secret JWT claims per Apple.
#[derive(Serialize)]
struct AppleClientSecretClaims {
    iss: String, // team_id
    iat: i64,
    exp: i64,
    aud: String, // "https://appleid.apple.com"
    sub: String, // client_id
}

impl OAuth2Provider for AppleOAuth2Provider {
    fn name() -> &'static str {
        "apple"
    }

    fn auth_url() -> &'static str {
        "https://appleid.apple.com/auth/authorize"
    }

    fn token_url() -> &'static str {
        "https://appleid.apple.com/auth/token"
    }

    fn default_scopes() -> &'static [&'static str] {
        &["name", "email"]
    }

    fn supports_userinfo() -> bool {
        false
    }

    async fn get_user_info(
        _http_client: &reqwest::Client,
        _access_token: &str,
    ) -> CornettiResult<OAuth2UserTransportData> {
        // Apple has no userinfo endpoint.
        // User data is in the id_token returned during the token exchange.
        // The service calls `decode_id_token` on the exchange id_token.
        Err(errors::auth_oauth2_errors::user_info_error()
            .with_internal_detail("Apple non supporta l'endpoint userinfo — usa l'id_token"))
    }
}

impl AppleOAuth2Provider {
    /// Generates the JWT client_secret for Apple.
    /// Requires `extra.key_id`, `extra.team_id`, `extra.private_key` in the provider conf.
    pub fn generate_client_secret(
        client_id: &str,
        extra: &toml::Value,
    ) -> CornettiResult<String> {
        let key_id = extra
            .get("key_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                errors::auth_oauth2_errors::provider_error()
                    .with_internal_detail("Apple extra.key_id is missing")
            })?;

        let team_id = extra
            .get("team_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                errors::auth_oauth2_errors::provider_error()
                    .with_internal_detail("Apple extra.team_id is missing")
            })?;

        let private_key = extra
            .get("private_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                errors::auth_oauth2_errors::provider_error()
                    .with_internal_detail("Apple extra.private_key is missing")
            })?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let claims = AppleClientSecretClaims {
            iss: team_id.to_string(),
            iat: now,
            exp: now + 3600, // 1 hour (Apple max: 6 months)
            aud: "https://appleid.apple.com".to_string(),
            sub: client_id.to_string(),
        };

        let header = Header {
            alg: Algorithm::ES256,
            kid: Some(key_id.to_string()),
            ..Default::default()
        };

        let encoding_key = EncodingKey::from_ec_pem(private_key.as_bytes()).map_err(|e| {
            errors::auth_oauth2_errors::provider_error()
                .with_internal_detail(format!("Apple private_key is invalid: {e}"))
        })?;

        jsonwebtoken::encode(&header, &claims, &encoding_key).map_err(|e| {
            errors::auth_oauth2_errors::provider_error()
                .with_internal_detail(format!("Error generating Apple client_secret: {e}"))
        })
    }

    /// Decodes an Apple id_token and returns the user data.
    ///
    /// # Security
    ///
    /// The id_token signature is **not** cryptographically verified: the
    /// token is accepted only because it was obtained through a direct call to
    /// Apple's token endpoint over TLS, which OIDC Core §3.1.3.7 allows as a
    /// substitute for signature verification. It must not be used on id_tokens
    /// from other channels.
    ///
    /// The `iss`, `aud` (must match `expected_client_id`) and `exp` claims are
    /// still validated.
    ///
    /// # Errors
    /// Returns `invalid_id_token` if the token is malformed, expired, or
    /// intended for a different client.
    pub fn decode_id_token(
        id_token: &str,
        expected_client_id: &str,
    ) -> CornettiResult<OAuth2UserTransportData> {
        let decoded = jsonwebtoken::dangerous::insecure_decode::<AppleIdTokenClaims>(id_token)
            .map_err(|e| {
                errors::auth_oauth2_errors::invalid_id_token()
                    .with_internal_detail(format!("Apple id_token decode error: {e}"))
            })?;

        let claims = decoded.claims;

        if let Some(iss) = claims.iss.as_deref()
            && iss != APPLE_ISSUER
        {
            return Err(errors::auth_oauth2_errors::invalid_id_token()
                .with_internal_detail(format!("Unexpected Apple id_token issuer: {iss}")));
        }

        if claims.aud.as_deref() != Some(expected_client_id) {
            return Err(errors::auth_oauth2_errors::invalid_id_token()
                .with_internal_detail("Apple id_token audience does not match client_id"));
        }

        if let Some(exp) = claims.exp
            && exp < chrono::Utc::now().timestamp()
        {
            return Err(errors::auth_oauth2_errors::invalid_id_token()
                .with_internal_detail("Expired Apple id_token"));
        }

        let raw = serde_json::to_value(&claims).unwrap_or_default();

        let email_verified = claims.email_verified.as_ref().and_then(|v| match v {
            serde_json::Value::Bool(b) => Some(*b),
            serde_json::Value::String(s) => Some(s == "true"),
            _ => None,
        });

        Ok(OAuth2UserTransportData {
            provider: "apple".to_string(),
            provider_user_id: claims.sub,
            email: claims.email,
            email_verified,
            name: claims.name,
            avatar_url: None,
            raw_data: raw,
        })
    }
}
