use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;
#[cfg(feature = "server")]
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Deserialize))]
pub enum Tipo {
    #[serde(rename = "telescopio")]
    Telescopio,
    #[serde(rename = "barlow_correttore")]
    BarlowCorrettore,
    #[serde(rename = "torretta")]
    Torretta,
    #[serde(rename = "binocolo")]
    Binocolo,
    #[serde(rename = "oculare")]
    Oculare,
    #[serde(rename = "cercatore")]
    Cercatore,
    #[serde(rename = "altro")]
    Altro,
}

impl Default for Tipo {
    fn default() -> Self {
        Self::Telescopio
    }
}

impl fmt::Display for Tipo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Telescopio => write!(f, "telescopio"),
            Self::BarlowCorrettore => write!(f, "barlow_correttore"),
            Self::Torretta => write!(f, "torretta"),
            Self::Binocolo => write!(f, "binocolo"),
            Self::Oculare => write!(f, "oculare"),
            Self::Cercatore => write!(f, "cercatore"),
            Self::Altro => write!(f, "altro"),
        }
    }
}

#[cfg(feature = "server")]
impl<'de> Deserialize<'de> for Tipo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_lowercase().as_str() {
            "telescopio" => Self::Telescopio,
            "barlow_correttore" | "barlow" | "correttore" => Self::BarlowCorrettore,
            "torretta" => Self::Torretta,
            "binocolo" => Self::Binocolo,
            "oculare" => Self::Oculare,
            "cercatore" => Self::Cercatore,
            "altro" => Self::Altro,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "client", derive(Deserialize, Clone))]
pub struct Strumentazione {
    pub id: Option<String>,
    pub tipo: Tipo,
    pub marca: Option<String>,
    pub modello: Option<String>,
    pub altro_tipo_personalizzato: Option<String>,
    pub altro_descr_estesa: Option<String>,
    pub diametro: Option<f64>,
    pub focale: Option<f64>,
    pub fattore_ingrandimento: Option<f64>,
    pub fov: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct StrumentazioneCreate {
    pub tipo: Tipo,
    #[serde(default)]
    pub marca: Option<String>,
    #[serde(default)]
    pub modello: Option<String>,
    #[serde(default)]
    pub altro_tipo_personalizzato: Option<String>,
    #[serde(default)]
    pub altro_descr_estesa: Option<String>,
    #[serde(default)]
    pub diametro: Option<f64>,
    #[serde(default)]
    pub focale: Option<f64>,
    #[serde(default)]
    pub fattore_ingrandimento: Option<f64>,
    #[serde(default)]
    pub fov: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema, Validate))]
#[cfg_attr(feature = "client", derive(Serialize, Clone))]
pub struct StrumentazioneUpdate {
    pub tipo: Tipo,
    #[serde(default)]
    pub marca: Option<String>,
    #[serde(default)]
    pub modello: Option<String>,
    #[serde(default)]
    pub altro_tipo_personalizzato: Option<String>,
    #[serde(default)]
    pub altro_descr_estesa: Option<String>,
    #[serde(default)]
    pub diametro: Option<f64>,
    #[serde(default)]
    pub focale: Option<f64>,
    #[serde(default)]
    pub fattore_ingrandimento: Option<f64>,
    #[serde(default)]
    pub fov: Option<f64>,
}
