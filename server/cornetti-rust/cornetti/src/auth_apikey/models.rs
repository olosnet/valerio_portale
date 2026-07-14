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

impl Default for AuthApiKeyStored {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_api_key_stored_new_has_id() {
        let stored = AuthApiKeyStored::new();
        assert!(!stored.id.is_empty());
    }

    #[test]
    fn auth_api_key_stored_new_default_false() {
        let stored = AuthApiKeyStored::new();
        assert!(!stored.default);
    }

    #[test]
    fn auth_api_key_stored_new_enabled_true() {
        let stored = AuthApiKeyStored::new();
        assert!(stored.enabled);
    }

    #[test]
    fn auth_api_key_stored_new_has_created() {
        let stored = AuthApiKeyStored::new();
        assert!(stored.created.is_some());
    }

    #[test]
    fn auth_api_key_stored_new_unique_id() {
        let s1 = AuthApiKeyStored::new();
        let s2 = AuthApiKeyStored::new();
        assert_ne!(s1.id, s2.id);
    }

    #[test]
    fn auth_api_key_create_response_new() {
        let stored = AuthApiKeyStored::new();
        let api_key = AuthApiKey::from(stored.clone());
        let resp = AuthApiKeyCreateResponse::new(api_key, "generated_key".into());
        assert_eq!(resp.generated_api_key, "generated_key");
        assert_eq!(resp.item.id, stored.id);
    }

    #[test]
    fn from_auth_api_key_stored_to_auth_api_key() {
        let stored = AuthApiKeyStored {
            id: "id1".into(), app_id: "app1".into(), name: "test".into(),
            resource_id: Some("res1".into()), key: "hashed".into(),
            note: Some("note1".into()), enabled: true, default: false,
            created: Some(chrono::Utc::now()), modified: chrono::Utc::now(),
        };
        let api_key = AuthApiKey::from(stored);
        assert_eq!(api_key.id, "id1");
        assert_eq!(api_key.app_id, "app1");
        assert_eq!(api_key.name, "test");
        assert_eq!(api_key.resource_id, Some("res1".into()));
        assert_eq!(api_key.note, Some("note1".into()));
    }

    #[test]
    fn from_auth_api_key_update_to_update_data() {
        let update = AuthApiKeyUpdate {
            name: "updated".into(),
            resource_id: Some("r2".into()),
            note: Some("note".into()),
            enabled: false,
        };
        let data = AuthApiKeyUpdateData::from(update);
        assert_eq!(data.name, "updated");
        assert_eq!(data.resource_id, Some("r2".into()));
        assert_eq!(data.note, Some("note".into()));
        assert!(!data.enabled);
    }

    #[test]
    fn auth_api_key_stored_new_app_id_empty() {
        let stored = AuthApiKeyStored::new();
        assert_eq!(stored.app_id, "");
    }

    #[test]
    fn auth_api_key_stored_new_key_empty() {
        let stored = AuthApiKeyStored::new();
        assert_eq!(stored.key, "");
    }
}
