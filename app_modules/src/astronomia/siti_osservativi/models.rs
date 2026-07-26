use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;
#[cfg(feature = "server")]
use validator::Validate;

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Deserialize, Clone, PartialEq))]
pub struct SitoOsservativo {
    pub id: Option<String>,
    pub nome: String,
    pub longitudine: f64,
    pub latitudine: f64,
    pub altitudine: f64,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct SitoOsservativoCreate {
    pub nome: String,
    pub longitudine: f64,
    pub latitudine: f64,
    pub altitudine: f64,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct SitoOsservativoUpdate {
    pub nome: String,
    pub longitudine: f64,
    pub latitudine: f64,
    pub altitudine: f64,
    pub timezone: Option<String>,
}
