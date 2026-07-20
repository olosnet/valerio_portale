use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct EnumItem {
    pub id: Option<String>,
    pub category: String,
    pub value: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct EnumCreate {
    pub category: String,
    pub value: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct EnumUpdate {
    pub category: String,
    pub value: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct EnumListQuery {
    pub category: Option<String>,
}