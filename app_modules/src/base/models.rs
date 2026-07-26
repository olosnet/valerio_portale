#[cfg(feature = "server")]
pub use cornetti::auth::models::AuthorizationPermission;

#[cfg(not(feature = "server"))]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "server"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorizationPermission {
    pub read: bool,
    pub create: bool,
    pub modify: bool,
    pub delete: bool,
}
