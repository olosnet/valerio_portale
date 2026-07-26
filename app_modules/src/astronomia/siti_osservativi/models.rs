use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct SitoOsservativo {
    pub id: Option<String>,
    pub nome: String,
    pub longitudine: f64,
    pub latitudine: f64,
    pub altitudine: f64,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SitoOsservativoCreate {
    pub nome: String,
    pub longitudine: f64,
    pub latitudine: f64,
    pub altitudine: f64,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SitoOsservativoUpdate {
    pub nome: String,
    pub longitudine: f64,
    pub latitudine: f64,
    pub altitudine: f64,
    pub timezone: Option<String>,
}
