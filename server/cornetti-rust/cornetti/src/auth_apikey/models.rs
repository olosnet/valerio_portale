use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Public-facing API key representation (sensitive data excluded).
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct AuthApiKey {
    /// Unique key identifier.
    pub id: String,
    /// Application identifier.
    pub app_id: String,
    /// Human-readable name.
    pub name: String,
    /// Optional associated resource identifier.
    pub resource_id: Option<String>,
    /// Optional note.
    pub note: Option<String>,
    /// Whether the key is currently enabled.
    pub enabled: bool,
    /// Whether this is a default key (cannot be modified or deleted).
    pub default: bool,
    /// Creation timestamp.
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    /// Last modification timestamp.
    pub modified: chrono::DateTime<chrono::Utc>,
}

/// Response returned when creating a new API key, includes the generated key value.
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct AuthApiKeyCreateResponse {
    /// The created API key metadata.
    pub item: AuthApiKey,
    /// The generated plain-text API key value (only returned once).
    pub generated_api_key: String,
}

impl AuthApiKeyCreateResponse {
    /// Creates a new response wrapping the created key and its generated value.
    pub fn new(item: AuthApiKey, generated_api_key: String) -> Self {
        Self {
            item,
            generated_api_key,
        }
    }
}

/// Request body for creating an API key.
#[derive(Debug, Deserialize, Clone, ToSchema, Validate)]
pub struct AuthApiKeyCreate {
    /// Human-readable key name (required).
    #[validate(length(min = 1))]
    pub name: String,
    /// Application identifier (required).
    #[validate(length(min = 1))]
    pub app_id: String,
    /// Optional associated resource identifier.
    pub resource_id: Option<String>,
    /// Optional note.
    pub note: Option<String>,
    /// Whether the key should be enabled.
    pub enabled: bool,
}

/// Request body for updating an API key.
#[derive(Debug, Deserialize, Clone, ToSchema, Validate)]
pub struct AuthApiKeyUpdate {
    /// Updated name (required).
    #[validate(length(min = 1))]
    pub name: String,
    /// Optional associated resource identifier.
    pub resource_id: Option<String>,
    /// Optional note.
    pub note: Option<String>,
    /// Whether the key is enabled.
    pub enabled: bool,
}

/// Internal storage representation including the hashed key value.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthApiKeyStored {
    /// Unique key identifier.
    pub id: String,
    /// Application identifier.
    pub app_id: String,
    /// Human-readable name.
    pub name: String,
    /// Optional associated resource identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// Argon2-hashed API key value.
    pub key: String,
    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Whether the key is enabled.
    pub enabled: bool,
    /// Whether this is a default key.
    pub default: bool,
    /// Creation timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    /// Last modification timestamp.
    pub modified: chrono::DateTime<chrono::Utc>,
}

/// Data for updating an existing stored API key.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthApiKeyUpdateData {
    pub name: String,
    pub resource_id: Option<String>,
    pub note: Option<String>,
    pub enabled: bool,
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl AuthApiKeyStored {
    /// Creates a new stored key with a generated UUID and default values.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            app_id: String::new(),
            name: String::new(),
            resource_id: None,
            key: String::new(),
            note: None,
            enabled: true,
            default: false,
            created: Some(chrono::Utc::now()),
            modified: chrono::Utc::now(),
        }
    }
}

impl From<AuthApiKeyStored> for AuthApiKey {
    /// Converts a stored representation to the public-facing representation,
    /// omitting the hashed key value.
    fn from(model: AuthApiKeyStored) -> Self {
        Self {
            id: model.id,
            app_id: model.app_id,
            name: model.name,
            resource_id: model.resource_id,
            note: model.note,
            enabled: model.enabled,
            default: model.default,
            created: model.created,
            modified: model.modified,
        }
    }
}

impl From<AuthApiKeyUpdate> for AuthApiKeyUpdateData {
    fn from(value: AuthApiKeyUpdate) -> Self {
        Self {
            name: value.name,
            resource_id: value.resource_id,
            note: value.note,
            enabled: value.enabled,
            modified: chrono::Utc::now(),
        }
    }
}
