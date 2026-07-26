use std::collections::HashMap;

use crate::base::models::AuthorizationPermission;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;
#[cfg(feature = "server")]
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Deserialize, Clone))]
pub struct UserIdentity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub surname: Option<String>,
    pub email: Option<String>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub last_access: Option<chrono::DateTime<chrono::Utc>>,
    pub profile_image: String,
    pub enabled: bool,
    pub default: bool,
    pub user_type: u8,
    pub groups_ids: Vec<String>,
    #[serde(default)]
    pub permissions: HashMap<String, AuthorizationPermission>,
}

#[cfg(feature = "server")]
impl UserIdentity {
    pub fn from_user_and_permissions(
        user: crate::base::users::models::User,
        permissions: HashMap<String, AuthorizationPermission>,
    ) -> Self {
        Self {
            id: user.id,
            name: user.name,
            surname: user.surname,
            email: user.email,
            created: user.created,
            modified: user.modified,
            last_access: user.last_access,
            profile_image: user.profile_image,
            enabled: user.enabled,
            default: user.default,
            user_type: user.user_type,
            groups_ids: user.groups_ids,
            permissions,
        }
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct UserIdentityUpdate {
    pub name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct UserIdentityUpdatePassword {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[cfg(feature = "server")]
impl Validate for UserIdentityUpdatePassword {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.old_password.is_empty() {
            let mut err = ValidationError::new("required");
            err.message = Some("old_password is required".into());
            errors.add("old_password", err);
        }

        if self.new_password.len() < 8 {
            let mut err = ValidationError::new("length");
            err.message = Some("new_password must be at least 8 characters long".into());
            errors.add("new_password", err);
        }

        if self.new_password != self.confirm_password {
            let mut err = ValidationError::new("mismatch");
            err.message = Some("Passwords do not match".into());
            errors.add("confirm_password", err);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
