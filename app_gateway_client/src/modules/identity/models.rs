use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::modules::base::models::AuthorizationPermission;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserIdentity {
    #[serde(rename = "_id")]
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
    pub permissions: HashMap<String, AuthorizationPermission>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserIdentityUpdate {
    pub name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserIdentityUpdatePassword {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}
