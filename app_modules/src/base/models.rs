use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorizationPermission {
    pub read: bool,
    pub create: bool,
    pub modify: bool,
    pub delete: bool,
}
