use std::collections::HashMap;

use cornetti::auth::models::AuthorizationPermission;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, ToSchema)]
pub struct UserIdentity {
    pub _id: Option<String>,
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
    pub permissions: HashMap<String, AuthorizationPermission>,
}

impl UserIdentity {
    pub fn from_user_and_permissions(
        user: crate::base::users::models::User,
        permissions: HashMap<String, AuthorizationPermission>,
    ) -> Self {
        Self {
            _id: user._id,
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

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UserIdentityUpdate {
    pub name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserIdentityUpdatePassword {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

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
