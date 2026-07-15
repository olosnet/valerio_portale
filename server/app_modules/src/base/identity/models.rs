use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

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
