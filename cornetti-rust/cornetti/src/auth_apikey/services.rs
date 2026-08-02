use std::sync::Arc;

use validator::Validate;

use crate::errors;
use crate::core::models::CornettiResult;

use super::{
    helpers::{
        extract_key_id, generate_api_key_value, hash_api_key, normalize_api_key_value,
        verify_api_key,
    },
    models::{
        AuthApiKey, AuthApiKeyCreate, AuthApiKeyCreateResponse, AuthApiKeyStored, AuthApiKeyUpdate,
        AuthApiKeyUpdateData,
    },
    traits::AuthApiKeyRepositoryTrait,
};

/// Service for CRUD operations on API keys.
pub struct AuthApiKeyService {
    tenant_id: String,
    repository: Box<dyn AuthApiKeyRepositoryTrait>,
}

impl AuthApiKeyService {
    /// Creates a new service for the given tenant using the provided repository.
    pub fn new(tenant_id: &str, repository: Box<dyn AuthApiKeyRepositoryTrait>) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            repository,
        }
    }

    /// Lists all API keys for the tenant.
    ///
    /// # Errors
    ///
    /// Returns a `CornettiError` if the repository fails.
    pub async fn list_api_keys(&self) -> CornettiResult<Vec<AuthApiKey>> {
        Ok(self
            .repository
            .list(&self.tenant_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Retrieves a single API key by ID.
    ///
    /// # Errors
    ///
    /// Returns 404 if the key is not found.
    pub async fn get_api_key(&self, key_id: &str) -> CornettiResult<AuthApiKey> {
        Ok(self
            .repository
            .get(&self.tenant_id, key_id.to_string())
            .await?
            .into())
    }

    /// Creates a new API key.
    ///
    /// The plain-text key value is returned only once in `generated_api_key`.
    ///
    /// # Errors
    ///
    /// Returns 400 if validation fails, or 500 if hashing fails.
    pub async fn create_api_key(
        &self,
        api_key_create: AuthApiKeyCreate,
    ) -> CornettiResult<AuthApiKeyCreateResponse> {
        api_key_create.validate()?;

        let mut model = AuthApiKeyStored::new();
        model.app_id = api_key_create.app_id;
        model.name = api_key_create.name;
        model.resource_id = api_key_create.resource_id;
        model.note = api_key_create.note;
        model.enabled = api_key_create.enabled;
        model.key = hash_api_key(&generate_api_key_value(&model.id))?;

        let created = self.repository.create(&self.tenant_id, model).await?;
        let generated = generate_api_key_value(&created.id);

        Ok(AuthApiKeyCreateResponse::new(created.into(), generated))
    }

    /// Updates an existing API key.
    ///
    /// Default keys cannot be modified.
    ///
    /// # Errors
    ///
    /// Returns 400 if validation fails or the key is marked as default.
    pub async fn update_api_key(
        &self,
        key_id: &str,
        api_key_update: AuthApiKeyUpdate,
    ) -> CornettiResult<AuthApiKey> {
        api_key_update.validate()?;

        let current = self
            .repository
            .get(&self.tenant_id, key_id.to_string())
            .await?;
        if current.default {
            return Err(errors::bad_request::validation_error()
                .with_internal_detail("Default api keys cannot be modified"));
        }

        let update_model: AuthApiKeyUpdateData = api_key_update.into();
        Ok(self
            .repository
            .update(&self.tenant_id, key_id.to_string(), update_model)
            .await?
            .into())
    }

    /// Deletes an API key.
    ///
    /// Default keys cannot be deleted.
    ///
    /// # Errors
    ///
    /// Returns 400 if the key is marked as default.
    /// Returns 404 if the key is not found.
    pub async fn delete_api_key(&self, key_id: &str) -> CornettiResult<()> {
        let current = self
            .repository
            .get(&self.tenant_id, key_id.to_string())
            .await?;

        if current.default {
            return Err(errors::bad_request::validation_error()
                .with_internal_detail("Default api keys cannot be deleted"));
        }

        self.repository.delete(&self.tenant_id, key_id.to_string()).await
    }
}

/// Authentication service that validates API key values against stored keys.
pub struct AuthApiKeyAuthService {
    #[allow(dead_code)]
    tenant_id: String,
    app_id: String,
    repository: Arc<dyn AuthApiKeyRepositoryTrait>,
}

impl AuthApiKeyAuthService {
    /// Creates a new authentication service.
    pub fn new(
        tenant_id: &str,
        app_id: &str,
        repository: Arc<dyn AuthApiKeyRepositoryTrait>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            app_id: app_id.to_string(),
            repository,
        }
    }

    /// Authenticates a raw API key value.
    ///
    /// The value may optionally include an `ApiKey` scheme prefix.
    ///
    /// # Errors
    ///
    /// Returns 401 if the key is missing, invalid, disabled, or does not belong
    /// to the configured application.
    pub async fn authenticate(&self, api_key_value: &str) -> CornettiResult<AuthApiKey> {
        let api_key_value = normalize_api_key_value(api_key_value).ok_or_else(|| {
            errors::authentication::custom_auth_error().with_internal_detail("Invalid API key")
        })?;

        let key_id = extract_key_id(api_key_value).ok_or_else(|| {
            errors::authentication::custom_auth_error().with_internal_detail("Invalid API key")
        })?;

        let model = match self.repository.find(key_id.to_string()).await? {
            Some(model) => model,
            None => {
                return Err(errors::authentication::custom_auth_error()
                    .with_internal_detail("Invalid API key"));
            }
        };

        if model.app_id != self.app_id {
            return Err(errors::authentication::custom_auth_error()
                .with_internal_detail("Invalid API key"));
        }

        if !model.enabled {
            return Err(errors::authentication::custom_auth_error()
                .with_internal_detail("API key disabled"));
        }

        if !verify_api_key(&model.key, api_key_value)? {
            return Err(errors::authentication::custom_auth_error()
                .with_internal_detail("Invalid API key"));
        }

        Ok(model.into())
    }
}
