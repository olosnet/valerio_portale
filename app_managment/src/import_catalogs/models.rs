pub use app_modules::astronomia::oggetti_astronomici::models::Costellazione;
use serde::{Deserialize, Serialize};

/// Internal representation of an astronomical object during import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCatalogEntry {
    pub cataloghi: Vec<CatalogEntry>,
    pub tipo: String,
    pub nome_comune: String,
    pub abbr_costellazione: Costellazione,
    pub coord_ar: String,
    pub coord_dec: String,
    pub mag_apparente: Option<f64>,
    pub dim_apparenti: Option<ImportDimApp>,
    pub note: String,
    pub multi: bool,

    /// Decimal RA for internal merge computations (not serialized).
    #[serde(skip)]
    pub ra_decimal: Option<f64>,
    /// Decimal Dec for internal merge computations (not serialized).
    #[serde(skip)]
    pub dec_decimal: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub catalog_id: String,
    pub catalog_nr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDimApp {
    pub secs_a: Option<i32>,
    pub secs_b: Option<i32>,
}
