use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupPermission {
    pub name: String,
    pub read: bool,
    pub create: bool,
    pub modify: bool,
    pub delete: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub created: String,
    pub modified: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub default: bool,
    pub permissions: Vec<GroupPermission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupCreate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<GroupPermission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupUpdate {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<GroupPermission>,
}
