use serde::{Deserialize, Serialize};

/// Full user with resolved permissions (used by login response)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserWithPermissions {
    #[serde(flatten)]
    pub user: app_modules::base::users::models::User,
    pub permissions: std::collections::HashMap<String, app_modules::base::models::AuthorizationPermission>,
}
