use std::sync::Arc;

use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
};

use crate::auth_oauth2::confs::{OAuth2AuthConf, OAuth2ProviderConf};
use crate::auth_oauth2::models::{OAuth2Metadata, OAuth2StateData};
use crate::auth_oauth2::providers::{apple, custom, BuiltinProvider};
use crate::auth_oauth2::traits::{OAuth2Provider, OAuth2SessionStore, OAuth2UserHandler};
use crate::core::helpers::sec::constant_time_eq;
use crate::core::models::CornettiResult;
use crate::errors;

/// Length of a PKCE code challenge with S256 method:
/// BASE64URL(SHA256(verifier)) without padding (RFC 7636 §4.2).
const PKCE_S256_CHALLENGE_LEN: usize = 43;

/// Allowed lengths for a PKCE code verifier (RFC 7636 §4.1).
const PKCE_VERIFIER_MIN_LEN: usize = 43;
const PKCE_VERIFIER_MAX_LEN: usize = 128;

/// OAuth2 client with auth URL and token URL set (oauth2 crate type-state pattern).
type BuiltBasicClient = BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

/// Store key for the CSRF state, bound to the provider that issued it.
///
/// Prevents a state obtained on one provider from being redeemed on
/// another provider's callback.
fn state_key(provider_name: &str, state: &str) -> String {
    format!("{provider_name}:{state}")
}

/// Validates a PKCE code challenge with S256 method (RFC 7636 §4.2):
/// 43 characters from the base64url alphabet, no padding.
///
/// # Errors
/// Returns `invalid_pkce_parameter` if the length or alphabet do not match.
fn validate_pkce_challenge(challenge: &str) -> CornettiResult<()> {
    let valid = challenge.len() == PKCE_S256_CHALLENGE_LEN
        && challenge
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');

    if !valid {
        return Err(
            errors::auth_oauth2_errors::invalid_pkce_parameter().with_internal_detail(
                "code_challenge non conforme a RFC 7636 S256 \
                 (43 caratteri, alfabeto base64url senza padding)",
            ),
        );
    }

    Ok(())
}

/// Validates a PKCE code verifier (RFC 7636 §4.1):
/// 43-128 characters from the `unreserved` alphabet.
///
/// # Errors
/// Returns `invalid_pkce_parameter` if the length or alphabet do not match.
fn validate_pkce_verifier(verifier: &str) -> CornettiResult<()> {
    let valid = (PKCE_VERIFIER_MIN_LEN..=PKCE_VERIFIER_MAX_LEN).contains(&verifier.len())
        && verifier
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'));

    if !valid {
        return Err(
            errors::auth_oauth2_errors::invalid_pkce_parameter().with_internal_detail(
                "code_verifier does not conform to RFC 7636 (43-128 chars, unreserved alphabet)",
            ),
        );
    }

    Ok(())
}

/// OAuth2 service — orchestrates the OAuth2 authorization flow end-to-end.
///
/// Handles:
/// 1. Authorization URL construction (with PKCE and anti-CSRF state)
/// 2. Authorization code exchange for tokens
/// 3. User info retrieval from the provider
/// 4. Local user lookup or creation via `OAuth2UserHandler`
///
/// Note: does NOT emit JWTs — it returns the data. JWT issuance and cookie
/// management is delegated to the actix integration (`actix/auth_oauth2/helpers.rs`).
pub struct OAuth2Service<U, T, S> {
    conf: Arc<OAuth2AuthConf>,
    user_handler: Arc<U>,
    http_client: reqwest::Client,
    state_store: Arc<S>,
    tenant_id: String,
    _marker: std::marker::PhantomData<T>,
}

impl<U, T, S> OAuth2Service<U, T, S> {
    /// Resolves the auth and token URLs for a provider: static URLs for
    /// built-ins, configured `auth_url`/`token_url` for custom providers.
    ///
    /// # Errors
    /// Returns `invalid_provider` if a custom provider has no URL configured
    /// (should be caught earlier by `OAuth2AuthConf::validate`).
    fn provider_urls(
        provider_conf: &OAuth2ProviderConf,
    ) -> CornettiResult<(String, String)> {
        match BuiltinProvider::from_name(&provider_conf.name) {
            Some(builtin) => Ok((
                builtin.auth_url().to_string(),
                builtin.token_url().to_string(),
            )),
            None => {
                let auth_url = provider_conf.auth_url.clone().ok_or_else(|| {
                    errors::auth_oauth2_errors::invalid_provider().with_internal_detail(
                        format!("Custom provider '{}' has no auth_url configured", provider_conf.name),
                    )
                })?;
                let token_url = provider_conf.token_url.clone().ok_or_else(|| {
                    errors::auth_oauth2_errors::invalid_provider().with_internal_detail(
                        format!("Custom provider '{}' has no token_url configured", provider_conf.name),
                    )
                })?;
                Ok((auth_url, token_url))
            }
        }
    }
}

impl<U, T, S> OAuth2Service<U, T, S>
where
    U: OAuth2UserHandler<T>,
    S: OAuth2SessionStore,
{
    /// Creates a new instance of the OAuth2 service.
    pub fn new(
        conf: Arc<OAuth2AuthConf>,
        user_handler: Arc<U>,
        state_store: Arc<S>,
        tenant_id: String,
    ) -> Self {
        Self {
            conf,
            user_handler,
            http_client: reqwest::Client::new(),
            state_store,
            tenant_id,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the OAuth2 configuration.
    pub fn conf(&self) -> &OAuth2AuthConf {
        &self.conf
    }

    /// Checks that OAuth2 authentication is enabled by configuration.
    ///
    /// # Errors
    /// Returns `auth_disabled` if `enable_auth` is `false`.
    fn ensure_enabled(&self) -> CornettiResult<()> {
        if !self.conf.enable_auth {
            return Err(errors::auth_oauth2_errors::auth_disabled()
                .with_internal_detail("OAuth2 disabled by configuration"));
        }
        Ok(())
    }

    /// Builds the `BasicClient` for the given provider, with the redirect_uri set.
    ///
    /// # Errors
    /// Returns `provider_error` if the auth/token URL or redirect URI are invalid.
    fn build_client(
        &self,
        provider_conf: &OAuth2ProviderConf,
    ) -> CornettiResult<BuiltBasicClient> {
        let client_id = ClientId::new(provider_conf.client_id.clone());
        let client_secret = ClientSecret::new(provider_conf.client_secret.clone());

        let (auth_url_str, token_url_str) = Self::provider_urls(provider_conf)?;

        let auth_url = AuthUrl::new(auth_url_str).map_err(|e| {
            errors::auth_oauth2_errors::provider_error()
                .with_internal_detail(format!("Invalid auth URL: {e}"))
        })?;

        let token_url = TokenUrl::new(token_url_str).map_err(|e| {
            errors::auth_oauth2_errors::provider_error()
                .with_internal_detail(format!("Invalid token URL: {e}"))
        })?;

        let client = BasicClient::new(client_id)
            .set_client_secret(client_secret)
            .set_auth_uri(auth_url)
            .set_token_uri(token_url);

        let redirect_uri = RedirectUrl::new(provider_conf.redirect_uri.clone()).map_err(|e| {
            errors::auth_oauth2_errors::provider_error()
                .with_internal_detail(format!("Invalid redirect URI: {e}"))
        })?;

        Ok(client.set_redirect_uri(redirect_uri))
    }

    /// Builds the authorization URL for the requested provider.
    ///
    /// Returns `(auth_url, csrf_state)`. The state is saved in the
    /// `OAuth2SessionStore` (one-shot) for verification in the callback.
    ///
    /// `client_pkce_challenge` selects who manages PKCE:
    /// - `None` → the server generates challenge and verifier, storing the
    ///   verifier. Suitable for the web flow, where the binding with the
    ///   victim's browser is guaranteed by the state cookie.
    /// - `Some(challenge)` → the verifier stays on the client and is never
    ///   transmitted to the server, which only saves the state. **Required in
    ///   the mobile flow**: anyone intercepting the redirect gets code and
    ///   state, but without the verifier cannot complete the exchange.
    ///
    /// # Security
    ///
    /// The endpoint that exposes this method is typically unauthenticated and
    /// writes an entry to the store on every call (with TTL
    /// `conf.state_ttl_secs`): it should be protected by rate limiting at the
    /// application level.
    ///
    /// # Errors
    /// - `auth_disabled`: OAuth2 not enabled by configuration
    /// - `invalid_provider`: provider not configured or not built-in
    /// - `invalid_pkce_parameter`: `client_pkce_challenge` does not conform to RFC 7636
    pub async fn build_auth_url(
        &self,
        provider_name: &str,
        client_pkce_challenge: Option<&str>,
    ) -> CornettiResult<(String, String)> {
        self.ensure_enabled()?;

        let provider_conf = self
            .conf
            .find_provider(provider_name)
            .ok_or_else(|| {
                errors::auth_oauth2_errors::invalid_provider()
                    .with_internal_detail(format!("Provider '{provider_name}' is not configured"))
            })?;

        let client = self.build_client(provider_conf)?;

        // Scopes: built-ins get their defaults plus the configured ones;
        // custom providers use only the configured scopes (validated non-empty).
        let mut scopes: Vec<Scope> = match BuiltinProvider::from_name(provider_name) {
            Some(builtin) => builtin
                .default_scopes()
                .iter()
                .map(|s| Scope::new(s.to_string()))
                .collect(),
            None => Vec::new(),
        };
        for s in &provider_conf.scopes {
            scopes.push(Scope::new(s.clone()));
        }

        let mut request = client.authorize_url(CsrfToken::new_random).add_scopes(scopes);

        // PKCE: client-supplied challenge or server-generated pair.
        // With a client challenge the parameters must be added manually, because
        // `PkceCodeChallenge` does not expose a constructor from a raw string.
        let state_data = match client_pkce_challenge {
            Some(challenge) => {
                validate_pkce_challenge(challenge)?;
                request = request
                    .add_extra_param("code_challenge", challenge)
                    .add_extra_param("code_challenge_method", "S256");
                OAuth2StateData {
                    pkce_verifier: None,
                }
            }
            None => {
                let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
                request = request.set_pkce_challenge(pkce_challenge);
                OAuth2StateData {
                    pkce_verifier: Some(pkce_verifier.secret().to_string()),
                }
            }
        };

        let (auth_url, csrf_token) = request.url();

        // Save the flow state associated with the CSRF state (one-shot)
        let state = csrf_token.secret().clone();
        let payload = serde_json::to_string(&state_data)?;

        self.state_store
            .set_oauth2_state(
                &self.tenant_id,
                &state_key(provider_name, &state),
                payload,
                self.conf.state_ttl_secs,
            )
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::session_store_error()
                    .with_internal_detail(e.to_string())
            })?;

        Ok((auth_url.to_string(), state))
    }

    /// Handles the OAuth2 callback: exchanges the code, retrieves user info,
    /// looks up or creates the local user.
    ///
    /// `expected_state` is the expected CSRF state value (e.g. from a cookie in web
    /// mode). When `None` (API mode), verification happens only through the
    /// one-shot store: an unknown/expired state fails with `pkce_not_found`.
    ///
    /// `client_code_verifier` is the PKCE verifier generated by the client. It
    /// must be present if and only if the flow was started with a client
    /// `code_challenge`:
    ///
    /// | verifier in store | verifier from client | outcome |
    /// |---|---|---|
    /// | present | absent | server-side flow |
    /// | absent  | present | client-side flow |
    /// | absent  | absent | `pkce_mode_mismatch` |
    /// | present | present | `pkce_mode_mismatch` |
    ///
    /// The third case is what prevents anyone intercepting code and state from
    /// completing the exchange without possessing the verifier.
    ///
    /// Returns `(user, metadata)`.
    ///
    /// # Errors
    /// - `auth_disabled`: OAuth2 not enabled by configuration
    /// - `state_mismatch`: CSRF state does not match the expected value
    /// - `invalid_provider`: provider not configured
    /// - `pkce_not_found`: state unknown, expired, or already consumed
    /// - `pkce_mode_mismatch`: verifier missing or inconsistent with the started flow
    /// - `invalid_pkce_parameter`: verifier does not conform to RFC 7636
    /// - `token_exchange_error`: code exchange failed
    /// - `user_info_error`: user info retrieval failed
    /// - `user_not_found`: user does not exist locally and
    ///   `auto_register_users` is disabled
    pub async fn handle_callback(
        &self,
        provider_name: &str,
        code: String,
        state: String,
        expected_state: Option<&str>,
        client_code_verifier: Option<String>,
    ) -> CornettiResult<(T, OAuth2Metadata)> {
        self.ensure_enabled()?;

        if let Some(expected) = expected_state
            && !constant_time_eq(&state, expected)
        {
            return Err(errors::auth_oauth2_errors::state_mismatch()
                .with_internal_detail("CSRF state does not match"));
        }

        let provider_conf = self
            .conf
            .find_provider(provider_name)
            .ok_or_else(|| {
                errors::auth_oauth2_errors::invalid_provider()
                    .with_internal_detail(format!("Provider '{provider_name}' is not configured"))
            })?;

        let builtin = BuiltinProvider::from_name(provider_name);

        // Retrieve and remove (one-shot) the flow state associated with the state
        let state_payload = self
            .state_store
            .take_oauth2_state(&self.tenant_id, &state_key(provider_name, &state))
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::session_store_error()
                    .with_internal_detail(e.to_string())
            })?;

        let state_data: OAuth2StateData = serde_json::from_str(
            &state_payload.ok_or_else(errors::auth_oauth2_errors::pkce_not_found)?,
        )?;

        // PKCE verifier resolution: server-side or client-side, never both
        let pkce_verifier = match (state_data.pkce_verifier, client_code_verifier) {
            (Some(stored), None) => stored,
            (None, Some(from_client)) => {
                validate_pkce_verifier(&from_client)?;
                from_client
            }
            (None, None) => {
                return Err(errors::auth_oauth2_errors::pkce_mode_mismatch()
                    .with_internal_detail(
"The flow was started with client-side PKCE: \
                         code_verifier is required in the callback",
                    ));
            }
            (Some(_), Some(_)) => {
                return Err(errors::auth_oauth2_errors::pkce_mode_mismatch()
                    .with_internal_detail(
"The flow was started with server-side PKCE: \
                         code_verifier is not expected in the callback",
                    ));
            }
        };

        let pkce_verifier = PkceCodeVerifier::new(pkce_verifier);

        // Exchange the code for tokens and retrieve user data
        let (access_token, refresh_token, expires_at, scopes, user_data) =
            if builtin == Some(BuiltinProvider::Apple) {
                // Apple has no userinfo endpoint: user data is in the id_token.
                // The exchange is done manually to capture the id_token (the oauth2
                // crate does not expose extra fields beyond the standard ones).
                let (access_token, refresh_token, expires_at, scopes, id_token) = self
                    .exchange_apple_token(provider_conf, &code, &pkce_verifier)
                    .await?;

                let id_token = id_token.ok_or_else(|| {
                    errors::auth_oauth2_errors::user_info_error()
                        .with_internal_detail("Apple did not return an id_token")
                })?;

                let user_data = apple::AppleOAuth2Provider::decode_id_token(
                    &id_token,
                    &provider_conf.client_id,
                )?;

                (access_token, refresh_token, expires_at, scopes, user_data)
            } else {
                let client = self.build_client(provider_conf)?;

                let token_result = client
                    .exchange_code(AuthorizationCode::new(code))
                    .set_pkce_verifier(pkce_verifier)
                    .request_async(&self.http_client)
                    .await
                    .map_err(|e| {
                        errors::auth_oauth2_errors::token_exchange_error()
                            .with_internal_detail(format!("Token exchange failed: {e}"))
                    })?;

                let access_token = token_result.access_token().secret().clone();
                let refresh_token = token_result.refresh_token().map(|t| t.secret().clone());
                let expires_at = token_result.expires_in().map(|d| {
                    chrono::Utc::now() + chrono::Duration::from_std(d).unwrap_or_default()
                });
                let scopes: Vec<String> = token_result
                    .scopes()
                    .map(|s| s.iter().map(|s| s.to_string()).collect())
                    .unwrap_or_default();

                // User info: built-in providers parse their own shape, custom
                // providers use the standard OIDC /userinfo endpoint.
                let user_data = match builtin {
                    Some(builtin) => {
                        builtin.get_user_info(&self.http_client, &access_token).await?
                    }
                    None => {
                        custom::get_user_info(&self.http_client, &access_token, provider_conf)
                            .await?
                    }
                };

                (access_token, refresh_token, expires_at, scopes, user_data)
            };

        // Look up or create the local user
        let (user, metadata) = match self
            .user_handler
            .find_by_oauth2(&self.tenant_id, provider_name, &user_data.provider_user_id)
            .await?
        {
            Some((existing_user, existing_meta)) => {
                // Update metadata with the new tokens
                let now = chrono::Utc::now();
                let metadata = OAuth2Metadata {
                    access_token,
                    refresh_token: refresh_token.or(existing_meta.refresh_token),
                    expires_at,
                    scopes,
                    updated_at: now,
                    ..existing_meta
                };
                self.user_handler
                    .update_oauth2_metadata(&self.tenant_id, &metadata)
                    .await?;

                (existing_user, metadata)
            }
            None => {
                if !self.conf.auto_register_users {
                    return Err(errors::auth_oauth2_errors::user_not_found()
                        .with_internal_detail(format!(
                            "OAuth2 user '{}' from provider '{provider_name}' does not exist \
                             and auto-registration is disabled",
                            user_data.provider_user_id
                        )));
                }

                self.user_handler
                    .create_from_oauth2(&self.tenant_id, &user_data)
                    .await?
            }
        };

        Ok((user, metadata))
    }

    /// Exchanges the Apple authorization code with the token endpoint.
    ///
    /// The client_secret is an ES256-signed JWT generated by `generate_client_secret`.
    /// The id_token (user data) is captured from the raw response.
    ///
    /// # Errors
    /// Returns `token_exchange_error` if the call fails.
    async fn exchange_apple_token(
        &self,
        provider_conf: &OAuth2ProviderConf,
        code: &str,
        pkce_verifier: &PkceCodeVerifier,
    ) -> CornettiResult<(
        String,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
        Vec<String>,
        Option<String>,
    )> {
        let client_secret = apple::AppleOAuth2Provider::generate_client_secret(
            &provider_conf.client_id,
            provider_conf
                .extra
                .as_ref()
                .unwrap_or(&toml::Value::Table(toml::map::Map::new())),
        )?;

        let response = self
            .http_client
            .post(apple::AppleOAuth2Provider::token_url())
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", provider_conf.client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("redirect_uri", provider_conf.redirect_uri.as_str()),
                ("code_verifier", pkce_verifier.secret()),
            ])
            .send()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::token_exchange_error()
                    .with_internal_detail(e.to_string())
            })?;

        if !response.status().is_success() {
            return Err(errors::auth_oauth2_errors::token_exchange_error()
                .with_internal_detail(format!("Apple token exchange HTTP {}", response.status())));
        }

        #[derive(serde::Deserialize)]
        struct AppleTokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            id_token: Option<String>,
            expires_in: Option<u64>,
            scope: Option<String>,
        }

        let token: AppleTokenResponse = response
            .json()
            .await
            .map_err(|e| {
                errors::auth_oauth2_errors::token_exchange_error()
                    .with_internal_detail(e.to_string())
            })?;

        let expires_at = token
            .expires_in
            .map(|s| chrono::Utc::now() + chrono::Duration::seconds(s as i64));
        let scopes: Vec<String> = token
            .scope
            .unwrap_or_default()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Ok((
            token.access_token,
            token.refresh_token,
            expires_at,
            scopes,
            token.id_token,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::http_status::HttpStatus;

    /// Valid S256 challenge: 43 base64url characters without padding.
    const VALID_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    fn provider_conf(name: &str) -> OAuth2ProviderConf {
        OAuth2ProviderConf {
            name: name.into(),
            client_id: "id".into(),
            client_secret: "secret".into(),
            redirect_uri: "https://example.test/cb".into(),
            scopes: vec!["openid".into()],
            auth_url: None,
            token_url: None,
            userinfo_url: None,
            extra: None,
        }
    }

    #[test]
    fn state_key_binds_provider_and_state() {
        assert_eq!(state_key("google", "abc"), "google:abc");
        // A state issued on one provider does not collide with another provider's
        assert_ne!(state_key("google", "abc"), state_key("github", "abc"));
    }

    #[test]
    fn valid_challenge_accepted() {
        assert_eq!(VALID_CHALLENGE.len(), PKCE_S256_CHALLENGE_LEN);
        assert!(validate_pkce_challenge(VALID_CHALLENGE).is_ok());
    }

    #[test]
    fn invalid_challenge_length_rejected() {
        assert!(validate_pkce_challenge(&"a".repeat(42)).is_err());
        assert!(validate_pkce_challenge(&"a".repeat(44)).is_err());
        assert!(validate_pkce_challenge("").is_err());
    }

    #[test]
    fn non_base64url_challenge_rejected() {
        // '+' and '/' belong to standard base64, not base64url
        assert!(validate_pkce_challenge(&format!("{}+", "a".repeat(42))).is_err());
        assert!(validate_pkce_challenge(&format!("{}/", "a".repeat(42))).is_err());
        // padding not allowed
        assert!(validate_pkce_challenge(&format!("{}=", "a".repeat(42))).is_err());
    }

    #[test]
    fn invalid_challenge_has_status_400() {
        let err = validate_pkce_challenge("troppo-corto").unwrap_err();
        assert_eq!(err.status, HttpStatus::BadRequest);
        assert_eq!(err.corr_id, "BE_INVALID_PKCE_PARAMETER");
    }

    #[test]
    fn verifier_boundary_lengths_accepted() {
        assert!(validate_pkce_verifier(&"a".repeat(PKCE_VERIFIER_MIN_LEN)).is_ok());
        assert!(validate_pkce_verifier(&"a".repeat(PKCE_VERIFIER_MAX_LEN)).is_ok());
    }

    #[test]
    fn verifier_out_of_range_rejected() {
        assert!(validate_pkce_verifier(&"a".repeat(PKCE_VERIFIER_MIN_LEN - 1)).is_err());
        assert!(validate_pkce_verifier(&"a".repeat(PKCE_VERIFIER_MAX_LEN + 1)).is_err());
    }

    #[test]
    fn unreserved_verifier_accepted() {
        let verifier = format!("{}-._~", "a".repeat(43));
        assert!(validate_pkce_verifier(&verifier).is_ok());
    }

    #[test]
    fn non_unreserved_verifier_rejected() {
        assert!(validate_pkce_verifier(&format!("{}+", "a".repeat(43))).is_err());
        assert!(validate_pkce_verifier(&format!("{} ", "a".repeat(43))).is_err());
        assert!(validate_pkce_verifier(&format!("{}/", "a".repeat(43))).is_err());
    }

    #[test]
    fn state_data_round_trip_json() {
        let server_side = OAuth2StateData {
            pkce_verifier: Some("verifier".into()),
        };
        let json = serde_json::to_string(&server_side).unwrap();
        let back: OAuth2StateData = serde_json::from_str(&json).unwrap();
        assert_eq!(back.pkce_verifier.as_deref(), Some("verifier"));

        let client_side = OAuth2StateData {
            pkce_verifier: None,
        };
        let json = serde_json::to_string(&client_side).unwrap();
        let back: OAuth2StateData = serde_json::from_str(&json).unwrap();
        assert!(back.pkce_verifier.is_none());
    }

    #[test]
    fn provider_urls_uses_static_urls_for_builtin() {
        let mut p = provider_conf("google");
        p.auth_url = Some("https://evil.example.test/authorize".into());
        let (auth, token) = OAuth2Service::<(), (), ()>::provider_urls(&p).unwrap();
        assert_eq!(auth, "https://accounts.google.com/o/oauth2/v2/auth");
        assert_eq!(token, "https://oauth2.googleapis.com/token");
    }

    #[test]
    fn provider_urls_uses_config_urls_for_custom() {
        let mut p = provider_conf("kanidm");
        p.auth_url = Some("https://idm.example.test/oauth2/openid_connect/authorize".into());
        p.token_url = Some("https://idm.example.test/oauth2/openid_connect/token".into());
        let (auth, token) = OAuth2Service::<(), (), ()>::provider_urls(&p).unwrap();
        assert_eq!(auth, "https://idm.example.test/oauth2/openid_connect/authorize");
        assert_eq!(token, "https://idm.example.test/oauth2/openid_connect/token");
    }

    #[test]
    fn provider_urls_custom_missing_url_errors() {
        let p = provider_conf("kanidm");
        let err = OAuth2Service::<(), (), ()>::provider_urls(&p).unwrap_err();
        assert_eq!(err.corr_id, "BE_INVALID_PROVIDER");
    }
}
