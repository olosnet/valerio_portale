use serde::Serialize;

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct EnumValue {
    pub name: &'static str,
    pub value: &'static str,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Clone))]
pub struct StaticsResponse {
    pub tipo_strumentazione: Vec<EnumValue>,
    pub tipo_oggetto: Vec<EnumValue>,
    pub costellazioni: Vec<EnumValue>,
    pub timezones: Vec<EnumValue>,
}
