use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::{IntoParams, ToSchema};
#[cfg(feature = "server")]
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Deserialize, Clone, PartialEq))]
pub struct User {
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
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct UserCreate {
    pub name: String,
    pub surname: String,
    #[cfg_attr(feature = "server", validate(email(message = "Invalid email format")))]
    pub email: String,
    pub enabled: bool,
    pub groups_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct UserUpdate {
    pub name: String,
    pub surname: String,
    pub enabled: bool,
    #[cfg_attr(feature = "server", schema(value_type = Vec<String>))]
    pub groups_ids: Vec<String>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, IntoParams))]
#[cfg_attr(feature = "client", derive(Debug, Serialize, Clone))]
pub struct SetPasswordBody {
    #[cfg_attr(feature = "server", param(min_length = 8, pattern = "[a-z]*"))]
    pub password: String,
    #[cfg_attr(feature = "server", param(min_length = 8, pattern = "[a-z]*"))]
    pub confirm_password: String,
}

#[cfg(feature = "server")]
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
