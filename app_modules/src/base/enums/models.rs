use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::{IntoParams, ToSchema};
#[cfg(feature = "server")]
use validator::Validate;

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Deserialize, Clone))]
pub struct EnumItem {
    pub id: Option<String>,
    pub category: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct EnumCreate {
    pub category: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct EnumUpdate {
    pub category: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(IntoParams))]
#[cfg_attr(feature = "client", derive(Clone))]
pub struct EnumListQuery {
    pub category: Option<String>,
}
