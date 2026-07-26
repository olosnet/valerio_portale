use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EnumValue {
    pub name: &'static str,
    pub value: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StaticsResponse {
    pub tipo_strumentazione: Vec<EnumValue>,
    pub tipo_oggetto: Vec<EnumValue>,
    pub costellazioni: Vec<EnumValue>,
    pub timezones: Vec<EnumValue>,
}
