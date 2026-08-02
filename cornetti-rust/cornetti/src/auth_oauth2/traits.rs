use crate::auth_oauth2::models::{OAuth2Metadata, OAuth2UserTransportData};
use crate::core::models::CornettiResult;
use std::future::Future;

/// OAuth2 provider.
///
/// Built-in providers implement this trait. For custom providers,
/// the consumer can implement and register them.
pub trait OAuth2Provider: Send + Sync {
    /// Unique provider name (e.g. "google", "github").
    fn name() -> &'static str;

    /// Provider authorization URL.
    fn auth_url() -> &'static str;

    /// Provider token exchange URL.
    fn token_url() -> &'static str;

    /// Default scopes requested from the provider.
    fn default_scopes() -> &'static [&'static str];

    /// Whether the provider exposes a userinfo endpoint.
    ///
    /// Apple does not have one: user data is in the id_token returned during
    /// the token exchange, so it overrides this to `false`.
    fn supports_userinfo() -> bool {
        true
    }

    /// Fetches the authenticated user's data from the provider,
    /// using the access token obtained in the token exchange.
    ///
    /// # Errors
    /// Returns `user_info_error` if the API call fails.
    fn get_user_info(
        http_client: &reqwest::Client,
        access_token: &str,
    ) -> impl Future<Output = CornettiResult<OAuth2UserTransportData>> + Send;
}

/// Handler for creating and retrieving local users + OAuth2 metadata.
///
/// The type `T` is the consumer's user model. The trait abstracts the
/// persistence logic — the consumer implements the lookup/creation in
/// their own database.
pub trait OAuth2UserHandler<T>: Send + Sync {
    /// Looks up a user already linked to an OAuth2 account.
    /// Returns the user and OAuth2 metadata if found.
    fn find_by_oauth2(
        &self,
        tenant_id: &str,
        provider: &str,
        provider_user_id: &str,
    ) -> impl Future<Output = CornettiResult<Option<(T, OAuth2Metadata)>>> + Send;

    /// Creates a new local user from the OAuth2 provider data
    /// and persists the associated metadata. Returns the created user and metadata.
    fn create_from_oauth2(
        &self,
        tenant_id: &str,
        user_data: &OAuth2UserTransportData,
    ) -> impl Future<Output = CornettiResult<(T, OAuth2Metadata)>> + Send;

    /// Updates existing OAuth2 metadata (e.g. after a token refresh).
    fn update_oauth2_metadata(
        &self,
        tenant_id: &str,
        metadata: &OAuth2Metadata,
    ) -> impl Future<Output = CornettiResult<()>> + Send;
}

/// Trait for extracting the identity (JWT subject) from the user model.
/// The consumer implements this trait on their user type T.
pub trait OAuth2Identity {
    /// Returns the subject (unique identity) for the JWT.
    fn subject(&self) -> String;
}

/// Store for temporary OAuth2 state (CSRF state + PKCE verifier).
///
/// Separate from `SessionStore` (JWT sessions): this holds ephemeral,
/// one-shot data. The consumer implements it on their own backend
/// (e.g. Redis) with a TTL.
///
/// # Security
///
/// `take_oauth2_state` **must** be destructive (atomic read-and-delete,
/// like `GETDEL` in Redis). An implementation that only reads without
/// deleting defeats the replay protection: the same state would remain
/// valid until its TTL expires and could be reused multiple times to
/// complete an authorization flow.
pub trait OAuth2SessionStore: Send + Sync {
    /// Stores the PKCE verifier associated with a CSRF state, with a TTL.
    fn set_oauth2_state(
        &self,
        tenant_id: &str,
        state_key: &str,
        pkce_verifier: String,
        ttl_secs: u64,
    ) -> impl Future<Output = CornettiResult<()>> + Send;

    /// Retrieves and removes the PKCE verifier associated with a state
    /// (one-shot, like GETDEL in Redis) — prevents replay of the CSRF state.
    ///
    /// # Security
    ///
    /// The removal must happen atomically together with the read: see the
    /// note on the trait itself.
    fn take_oauth2_state(
        &self,
        tenant_id: &str,
        state_key: &str,
    ) -> impl Future<Output = CornettiResult<Option<String>>> + Send;
}
