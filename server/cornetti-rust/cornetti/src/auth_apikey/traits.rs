use std::{future::Future, pin::Pin};

use crate::core::models::CornettiResult;

use super::models::{AuthApiKeyStored, AuthApiKeyUpdateData};

/// Repository trait for API key persistence.
///
/// Implementations must be `Send + Sync` for use in actix-web middlewares.
pub trait AuthApiKeyRepositoryTrait: Send + Sync {
    /// Lists all stored API keys for the given tenant.
    fn list(
        &self,
        tenant_id: &str,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<Vec<AuthApiKeyStored>>> + Send>>;

    /// Retrieves a single stored key by ID (must exist).
    fn get(
        &self,
        tenant_id: &str,
        key_id: String,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<AuthApiKeyStored>> + Send>>;

    /// Finds a stored key by ID, returning `None` if not found.
    fn find(
        &self,
        key_id: String,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<Option<AuthApiKeyStored>>> + Send>>;

    /// Creates a new stored API key.
    fn create(
        &self,
        tenant_id: &str,
        model: AuthApiKeyStored,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<AuthApiKeyStored>> + Send>>;

    /// Updates an existing stored API key.
    fn update(
        &self,
        tenant_id: &str,
        key_id: String,
        model: AuthApiKeyUpdateData,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<AuthApiKeyStored>> + Send>>;

    /// Deletes a stored API key by ID.
    fn delete(
        &self,
        tenant_id: &str,
        key_id: String,
    ) -> Pin<Box<dyn Future<Output = CornettiResult<()>> + Send>>;
}
