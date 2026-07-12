use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::astronomia::oggetti_astronomici::helpers::{validate_coord_ar, validate_coord_dec};

/// Tipologia di oggetto astronomico.
///
/// I codici brevi (GAL, OpC, ...) seguono le convenzioni usate nei cataloghi
/// NGC/IC (OpenNGC) e nella letteratura amatoriale. Le sottoclassi di galassie
/// (GAL_EL, GAL_SP, ...) estendono il codice base GAL per una classificazione
/// morfologica piu' fine (Hubble sequence, attività nucleare).
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub enum TipoOggetto {
    // --- Galassie (classificazione morfologica) ---
    #[serde(rename = "GAL")]
    Galassia,
    #[serde(rename = "GAL_EL")]
    GalassiaEllittica,
    #[serde(rename = "GAL_LN")]
    GalassiaLenticolare,
    #[serde(rename = "GAL_SP")]
    GalassiaSpirale,
    #[serde(rename = "GAL_SB")]
    GalassiaBarrata,
    #[serde(rename = "GAL_IR")]
    GalassiaIrregolare,
    #[serde(rename = "GAL_DW")]
    GalassiaNana,
    #[serde(rename = "GAL_PEC")]
    GalassiaPeculiare,
    #[serde(rename = "GAL_AGN")]
    GalassiaAttiva,
    // --- Ammassi ---
    #[serde(rename = "OpC")]
    AmmassoAperto,
    #[serde(rename = "GCl")]
    AmmassoGlobulare,
    // --- Nebulose ---
    #[serde(rename = "Neb")]
    Nebulosa,
    #[serde(rename = "EmN")]
    NebulosaEmissione,
    #[serde(rename = "RfN")]
    NebulosaRiflessione,
    #[serde(rename = "PN")]
    NebulosaPlanetaria,
    #[serde(rename = "SNR")]
    RestoSupernova,
    #[serde(rename = "HII")]
    RegioneHII,
    #[serde(rename = "DkNeb")]
    NebulosaOscura,
    // --- Ammassi di galassie ---
    #[serde(rename = "GCL")]
    AmmassoGalassie,
    #[serde(rename = "HCG")]
    GruppoGalassie,
    // --- Stelle ---
    #[serde(rename = "Star")]
    Stella,
    #[serde(rename = "2Star")]
    StellaDoppia,
    #[serde(rename = "Aster")]
    Asterismo,
    // --- Altro ---
    #[serde(rename = "StarCloud")]
    NubeStellare,
    #[serde(rename = "Neb+OpC")]
    NebulosaAmmasso,
    #[serde(rename = "QSO")]
    Quasar,
    #[serde(rename = "PSR")]
    Pulsar,
}

impl Default for TipoOggetto {
    fn default() -> Self {
        Self::Galassia
    }
}

impl fmt::Display for TipoOggetto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Galassia => write!(f, "GAL"),
            Self::GalassiaEllittica => write!(f, "GAL_EL"),
            Self::GalassiaLenticolare => write!(f, "GAL_LN"),
            Self::GalassiaSpirale => write!(f, "GAL_SP"),
            Self::GalassiaBarrata => write!(f, "GAL_SB"),
            Self::GalassiaIrregolare => write!(f, "GAL_IR"),
            Self::GalassiaNana => write!(f, "GAL_DW"),
            Self::GalassiaPeculiare => write!(f, "GAL_PEC"),
            Self::GalassiaAttiva => write!(f, "GAL_AGN"),
            Self::AmmassoAperto => write!(f, "OpC"),
            Self::AmmassoGlobulare => write!(f, "GCl"),
            Self::Nebulosa => write!(f, "Neb"),
            Self::NebulosaEmissione => write!(f, "EmN"),
            Self::NebulosaRiflessione => write!(f, "RfN"),
            Self::NebulosaPlanetaria => write!(f, "PN"),
            Self::RestoSupernova => write!(f, "SNR"),
            Self::RegioneHII => write!(f, "HII"),
            Self::NebulosaOscura => write!(f, "DkNeb"),
            Self::AmmassoGalassie => write!(f, "GCL"),
            Self::GruppoGalassie => write!(f, "HCG"),
            Self::Stella => write!(f, "Star"),
            Self::StellaDoppia => write!(f, "2Star"),
            Self::Asterismo => write!(f, "Aster"),
            Self::NubeStellare => write!(f, "StarCloud"),
            Self::NebulosaAmmasso => write!(f, "Neb+OpC"),
            Self::Quasar => write!(f, "QSO"),
            Self::Pulsar => write!(f, "PSR"),
        }
    }
}

impl<'de> Deserialize<'de> for TipoOggetto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "GAL" => Self::Galassia,
            "GAL_EL" | "GALE" => Self::GalassiaEllittica,
            "GAL_LN" | "GALS0" => Self::GalassiaLenticolare,
            "GAL_SP" | "GALS" => Self::GalassiaSpirale,
            "GAL_SB" | "GALSB" => Self::GalassiaBarrata,
            "GAL_IR" | "GALI" => Self::GalassiaIrregolare,
            "GAL_DW" | "GALD" => Self::GalassiaNana,
            "GAL_PEC" => Self::GalassiaPeculiare,
            "GAL_AGN" | "AGN" => Self::GalassiaAttiva,
            "OpC" => Self::AmmassoAperto,
            "GCl" => Self::AmmassoGlobulare,
            "GC" => Self::AmmassoGlobulare, // OpenNGC code
            "Neb" => Self::Nebulosa,
            "EmN" => Self::NebulosaEmissione,
            "RfN" => Self::NebulosaRiflessione,
            "PN" => Self::NebulosaPlanetaria,
            "SNR" => Self::RestoSupernova,
            "HII" => Self::RegioneHII,
            "DkNeb" => Self::NebulosaOscura,
            "GCL" | "GClust" => Self::AmmassoGalassie,
            "HCG" => Self::GruppoGalassie,
            "Star" | "*" => Self::Stella,
            "2Star" | "DS" | "**" => Self::StellaDoppia,
            "Aster" | "Ast" => Self::Asterismo,
            "StarCloud" => Self::NubeStellare,
            "Neb+OpC" | "OC+Neb" => Self::NebulosaAmmasso,
            "QSO" => Self::Quasar,
            "PSR" => Self::Pulsar,
            _ => Self::default(),
        })
    }
}

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
    pub tipo: TipoOggetto,
    pub nome_comune: String,
    pub abbr_costellazione: String,
    pub coord_ar: String,
    pub coord_dec: String,
    pub mag_apparente: Option<f64>,
    pub dim_apparenti: Option<DimensioniApparenti>,
    pub note: String,
    pub cataloghi: Vec<Catalogo>,
    pub multi: bool,
    pub imported: bool,
    pub image_filename: Option<String>,
    pub image_caption: Option<String>,
    pub image_fov: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct OggettoAstronomicoCreate {
    pub tipo: TipoOggetto,
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
    pub multi: bool,
    #[serde(default)]
    pub imported: bool,
    #[serde(default)]
    #[validate(nested)]
    pub cataloghi: Vec<CatalogoInput>,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct OggettoAstronomicoUpdate {
    pub tipo: TipoOggetto,
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
    pub multi: bool,
    #[serde(default)]
    pub imported: bool,
    #[serde(default)]
    #[validate(nested)]
    pub cataloghi: Vec<CatalogoInput>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OggettoAstronomicoImageUploadBody {
    pub caption: Option<String>,
}
