use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct GroupPermission {
    name: String,
    pub read: bool,
    pub create: bool,
    pub modify: bool,
    pub delete: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Group {
    pub id: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub default: bool,
    pub permissions: Vec<GroupPermission>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct GroupCreate {
    #[serde(default)]
    #[schema(required = true)]
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<GroupPermission>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct GroupUpdate {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<GroupPermission>,
}
