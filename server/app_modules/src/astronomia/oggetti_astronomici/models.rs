use crate::astronomia::oggetti_astronomici::helpers::{validate_coord_ar, validate_coord_dec};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Catalogo {
    pub catalog_id: String,
    pub catalog_nr: String,
    pub extended: String,
}

#[derive(Debug, Deserialize, ToSchema, Validate, Clone)]
pub struct CatalogoInput {
    pub catalog_id: String,
    pub catalog_nr: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DimensioniApparenti {
    pub secs_a: Option<i32>,
    pub secs_b: Option<i32>,
    pub dms_a: Option<String>,
    pub dms_b: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate, Clone)]
pub struct DimensioniApparentiInput {
    pub secs_a: Option<i32>,
    pub secs_b: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OggettoAstronomico {
    pub _id: Option<String>,
    pub tipo: String,
    pub nome_comune: String,
    pub abbr_costellazione: String,
    pub coord_ar: String,
    pub coord_dec: String,
    pub mag_apparente: Option<f64>,
    pub dim_apparenti: Option<DimensioniApparenti>,
    pub note: String,
    pub cataloghi: Vec<Catalogo>,
    pub image_filename: Option<String>,
    pub image_caption: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct OggettoAstronomicoCreate {
    pub tipo: String,
    #[serde(default)]
    pub nome_comune: String,
    #[serde(default)]
    pub abbr_costellazione: String,
    #[serde(default)]
    #[validate(custom(function = "validate_coord_ar"))]
    pub coord_ar: String,
    #[serde(default)]
    #[validate(custom(function = "validate_coord_dec"))]
    pub coord_dec: String,
    pub mag_apparente: Option<f64>,
    #[validate(nested)]
    pub dim_apparenti: Option<DimensioniApparentiInput>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    #[validate(nested)]
    pub cataloghi: Vec<CatalogoInput>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct OggettoAstronomicoUpdate {
    pub tipo: String,
    #[serde(default)]
    pub nome_comune: String,
    #[serde(default)]
    pub abbr_costellazione: String,
    #[serde(default)]
    #[validate(custom(function = "validate_coord_ar"))]
    pub coord_ar: String,
    #[serde(default)]
    #[validate(custom(function = "validate_coord_dec"))]
    pub coord_dec: String,
    pub mag_apparente: Option<f64>,
    #[validate(nested)]
    pub dim_apparenti: Option<DimensioniApparentiInput>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    #[validate(nested)]
    pub cataloghi: Vec<CatalogoInput>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OggettoAstronomicoImageUploadBody {
    pub caption: Option<String>,
}
