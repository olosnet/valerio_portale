use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, ToSchema)]
pub struct User {
    pub _id: Option<String>, //La descrizione dei singoli campi attualmente non è supportata
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
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UserCreate {
    pub name: String,
    pub surname: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub enabled: bool,
    pub groups_ids: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct UserUpdate {
    pub name: String,
    pub surname: String,
    pub enabled: bool,
    #[schema(value_type = Vec<String>)]
    pub groups_ids: Vec<String>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct SetPasswordBody {
    #[param(min_length = 8, pattern = "[a-z]*")]
    pub password: String,
    #[param(min_length = 8, pattern = "[a-z]*")]
    pub confirm_password: String,
}

impl Validate for SetPasswordBody {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        if self.password.len() < 8 {
            let mut err = ValidationError::new("length");
            err.message = Some("Password must be at least 8 characters long".into());
            errors.add("password", err);
        }

        if self.password != self.confirm_password {
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
