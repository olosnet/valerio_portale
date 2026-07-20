use serde::{Deserialize, Serialize};
use crate::modules::base::models::AuthorizationPermission;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Option<String>,
    pub name: Option<String>,
    pub surname: Option<String>,
    pub email: Option<String>,
    pub created: Option<String>,
    pub modified: String,
    pub last_access: Option<String>,
    pub profile_image: String,
    pub enabled: bool,
    pub default: bool,
    pub user_type: u8,
    pub groups_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserCreate {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub enabled: bool,
    pub groups_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserUpdate {
    pub name: String,
    pub surname: String,
    pub enabled: bool,
    pub groups_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetPasswordBody {
    pub password: String,
    pub confirm_password: String,
}

/// Full user with resolved permissions (used by login response)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserWithPermissions {
    #[serde(flatten)]
    pub user: User,
    pub permissions: std::collections::HashMap<String, AuthorizationPermission>,
}
