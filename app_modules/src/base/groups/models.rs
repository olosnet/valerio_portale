use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;
#[cfg(feature = "server")]
use validator::Validate;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(PartialEq))]
pub struct GroupPermission {
    pub name: String,
    pub read: bool,
    pub create: bool,
    pub modify: bool,
    pub delete: bool,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Deserialize, Clone, PartialEq))]
pub struct Group {
    pub id: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub default: bool,
    pub permissions: Vec<GroupPermission>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct GroupCreate {
    #[serde(default)]
    #[cfg_attr(feature = "server", schema(required = true))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<GroupPermission>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct GroupUpdate {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<GroupPermission>,
}
