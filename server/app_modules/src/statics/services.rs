use crate::statics::models::StaticsResponse;
use crate::statics::{COSTELLAZIONI, TIPO_OGGETTO, TIPO_STRUMENTAZIONE};

pub struct StaticsService;

impl StaticsService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_enum_values(&self) -> StaticsResponse {
        StaticsResponse {
            tipo_strumentazione: TIPO_STRUMENTAZIONE.to_vec(),
            tipo_oggetto: TIPO_OGGETTO.to_vec(),
            costellazioni: COSTELLAZIONI.to_vec(),
        }
    }
}
