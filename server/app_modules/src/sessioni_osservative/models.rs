use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Osservazione {
    pub uuid: String,
    pub note_osservazione: String,
    pub miglior_ingrandimento: i32,
    pub oggetti_id: Vec<String>,
    pub oggetto_id_ref: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct OsservazioneInput {
    pub note_osservazione: String,
    pub miglior_ingrandimento: i32,
    #[serde(default)]
    pub oggetti_id: Vec<String>,
    pub oggetto_id_ref: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct MisurazioneSqm {
    pub uuid: String,
    pub sqm: f64,
    pub temperatura: f64,
    pub umidita: f64,
    pub dataora_rilievo: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct MisurazioneSqmInput {
    pub sqm: f64,
    pub temperatura: f64,
    pub umidita: f64,
    pub dataora_rilievo: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct StrumentazioneSessione {
    pub uuid: String,
    pub tipo: String,
    pub marca: String,
    pub modello: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct StrumentazioneSessioneInput {
    pub tipo: String,
    pub marca: String,
    pub modello: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessioneOsservativa {
    pub _id: Option<String>,
    pub data: chrono::DateTime<chrono::Utc>,
    pub intro: String,
    pub outro: String,
    pub sito_osservativo_id: String,
    pub strumentazione: Vec<StrumentazioneSessione>,
    pub oggetti_osservati: Vec<Osservazione>,
    pub misurazioni_sqm: Vec<MisurazioneSqm>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SessioneOsservativaCreate {
    pub data: chrono::DateTime<chrono::Utc>,
    pub intro: String,
    pub outro: String,
    pub sito_osservativo_id: String,
    #[serde(default)]
    #[validate(nested)]
    pub strumentazione: Vec<StrumentazioneSessioneInput>,
    #[serde(default)]
    #[validate(nested)]
    pub oggetti_osservati: Vec<OsservazioneInput>,
    #[serde(default)]
    #[validate(nested)]
    pub misurazioni_sqm: Vec<MisurazioneSqmInput>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct SessioneOsservativaUpdate {
    pub data: chrono::DateTime<chrono::Utc>,
    pub intro: String,
    pub outro: String,
    pub sito_osservativo_id: String,
    #[serde(default)]
    #[validate(nested)]
    pub strumentazione: Vec<StrumentazioneSessioneInput>,
    #[serde(default)]
    #[validate(nested)]
    pub oggetti_osservati: Vec<OsservazioneInput>,
    #[serde(default)]
    #[validate(nested)]
    pub misurazioni_sqm: Vec<MisurazioneSqmInput>,
}
