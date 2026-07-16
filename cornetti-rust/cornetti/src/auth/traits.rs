use crate::{
    auth::models::{AuthorizationPermission, SessionStoreData},
    core::models::CornettiResult,
};

/// JWT token lifecycle: creation, validation, encoding, decoding.
pub trait BaseJwtToken {
    /// Creates a new token (access or refresh) for the given subject and session.
    fn new(
        conf: crate::auth::confs::JwtAuthConf,
        subject: String,
        session_id: String,
        refresh: bool,
    ) -> Self;
    /// Builds the JWT validation rules from the configuration.
    fn validator(conf: &crate::auth::confs::JwtAuthConf) -> jsonwebtoken::Validation;
    /// Encodes the token into a JWT string.
    fn encode(
        &self,
        conf: &crate::auth::confs::JwtAuthConf,
    ) -> Result<String, jsonwebtoken::errors::Error>;
    /// Decodes a JWT string and validates it against the configuration.
    fn decode(
        token: &str,
        conf: &crate::auth::confs::JwtAuthConf,
    ) -> Result<Self, jsonwebtoken::errors::Error>
    where
        Self: Sized;
}

/// Backend store for session tracking (access tokens, refresh tokens, sessions).
///
/// Implementations must be `Send + Sync` for use in actix-web middlewares.
pub trait SessionStore {
    /// Persists a token (access or refresh) in the store for the given tenant.
    fn add_token(
        &self,
        tenant_id: &str,
        claim: &SessionStoreData,
    ) -> impl std::future::Future<Output = CornettiResult<()>> + Send;

    /// Removes a specific auth token by JTI. Returns the number of keys removed.
    fn remove_auth_token(
        &self,
        tenant_id: &str,
        jti: &str,
    ) -> impl std::future::Future<Output = CornettiResult<usize>> + Send;
    /// Removes a specific refresh token by JTI. Returns the number of keys removed.
    fn remove_refresh_token(
        &self,
        tenant_id: &str,
        jti: &str,
    ) -> impl std::future::Future<Output = CornettiResult<usize>> + Send;
    /// Retrieves an auth token by JTI.
    fn get_auth_token(
        &self,
        tenant_id: &str,
        jti: &str,
    ) -> impl std::future::Future<Output = CornettiResult<Option<SessionStoreData>>> + Send;
    /// Retrieves a refresh token by JTI.
    fn get_refresh_token(
        &self,
        tenant_id: &str,
        jti: &str,
    ) -> impl std::future::Future<Output = CornettiResult<Option<SessionStoreData>>> + Send;

    /// Removes an entire session (auth + refresh tokens) for the given subject
    /// and session ID. Returns the number of keys removed.
    fn remove_session(
        &self,
        tenant_id: &str,
        sub: &str,
        session_id: &str,
    ) -> impl std::future::Future<Output = CornettiResult<usize>> + Send;
    /// Lists all active (non-expired) sessions for a subject.
    fn subject_sessions(
        &self,
        tenant_id: &str,
        sub: &str,
    ) -> impl std::future::Future<Output = CornettiResult<Vec<SessionStoreData>>> + Send;
    /// Clears all sessions for a subject. Returns the number of sessions cleared.
    fn clear_subject_sessions(
        &self,
        tenant_id: &str,
        sub: &str,
    ) -> impl std::future::Future<Output = CornettiResult<usize>> + Send;
}

/// Resolves authorization permissions for a given identity.
pub trait IdentityAuthorization {
    /// Returns a map of permission name → permission flags for the identity.
    fn get_identity_permissions(
        &self,
        tenant_id: &str,
        sub: &str,
    ) -> impl std::future::Future<
        Output = CornettiResult<std::collections::HashMap<String, AuthorizationPermission>>,
    > + Send;
}
