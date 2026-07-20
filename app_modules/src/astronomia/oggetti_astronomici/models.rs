use std::fmt;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::astronomia::oggetti_astronomici::helpers::{validate_coord_ar, validate_coord_dec};

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub enum Costellazione {
    #[serde(rename = "Sconosciuta")]
    Sconosciuta,
    #[serde(rename = "And")]
    Andromeda,
    #[serde(rename = "Ant")]
    MacchinaPneumatica,
    #[serde(rename = "Aps")]
    UccelloDelParadiso,
    #[serde(rename = "Aql")]
    Aquila,
    #[serde(rename = "Aqr")]
    Aquario,
    #[serde(rename = "Ara")]
    Altare,
    #[serde(rename = "Ari")]
    Ariete,
    #[serde(rename = "Aur")]
    Auriga,
    #[serde(rename = "Boo")]
    Boote,
    #[serde(rename = "Cae")]
    Bulino,
    #[serde(rename = "Cam")]
    Giraffa,
    #[serde(rename = "Cap")]
    Capricorno,
    #[serde(rename = "Car")]
    Carena,
    #[serde(rename = "Cas")]
    Cassiopea,
    #[serde(rename = "Cen")]
    Centauro,
    #[serde(rename = "Cep")]
    Cefeo,
    #[serde(rename = "Cet")]
    Balena,
    #[serde(rename = "Cha")]
    Camaleonte,
    #[serde(rename = "Cir")]
    Compasso,
    #[serde(rename = "CMa")]
    CaneMaggiore,
    #[serde(rename = "CMi")]
    CaneMinore,
    #[serde(rename = "Cnc")]
    Cancro,
    #[serde(rename = "Col")]
    Colomba,
    #[serde(rename = "Com")]
    ChiomaBerenice,
    #[serde(rename = "CrA")]
    CoronaAustrale,
    #[serde(rename = "CrB")]
    CoronaBoreale,
    #[serde(rename = "Crt")]
    Cratere,
    #[serde(rename = "Cru")]
    CroceDelSud,
    #[serde(rename = "Crv")]
    Corvo,
    #[serde(rename = "CVn")]
    CaniDaCaccia,
    #[serde(rename = "Cyg")]
    Cigno,
    #[serde(rename = "Del")]
    Delfino,
    #[serde(rename = "Dor")]
    Dorado,
    #[serde(rename = "Dra")]
    Dragone,
    #[serde(rename = "Equ")]
    Cavallino,
    #[serde(rename = "Eri")]
    Eridano,
    #[serde(rename = "For")]
    Fornace,
    #[serde(rename = "Gem")]
    Gemelli,
    #[serde(rename = "Gru")]
    Gru,
    #[serde(rename = "Her")]
    Ercole,
    #[serde(rename = "Hor")]
    Orologio,
    #[serde(rename = "Hya")]
    IdraFemmina,
    #[serde(rename = "Hyi")]
    IdraMaschio,
    #[serde(rename = "Ind")]
    Indiano,
    #[serde(rename = "Lac")]
    Lucertola,
    #[serde(rename = "Leo")]
    Leone,
    #[serde(rename = "Lep")]
    Lepre,
    #[serde(rename = "Lib")]
    Bilancia,
    #[serde(rename = "LMi")]
    LeoneMinore,
    #[serde(rename = "Lup")]
    Lupo,
    #[serde(rename = "Lyn")]
    Lince,
    #[serde(rename = "Lyr")]
    Lira,
    #[serde(rename = "Men")]
    Mensa,
    #[serde(rename = "Mic")]
    Microscopio,
    #[serde(rename = "Mon")]
    Unicorno,
    #[serde(rename = "Mus")]
    Mosca,
    #[serde(rename = "Nor")]
    Regolo,
    #[serde(rename = "Oct")]
    Ottante,
    #[serde(rename = "Oph")]
    Ofiuco,
    #[serde(rename = "Ori")]
    Orione,
    #[serde(rename = "Pav")]
    Pavone,
    #[serde(rename = "Peg")]
    Pegaso,
    #[serde(rename = "Per")]
    Perseo,
    #[serde(rename = "Phe")]
    Fenice,
    #[serde(rename = "Pic")]
    Pittore,
    #[serde(rename = "PsA")]
    PesceAustrale,
    #[serde(rename = "Psc")]
    Pesci,
    #[serde(rename = "Pup")]
    Poppa,
    #[serde(rename = "Pyx")]
    Bussola,
    #[serde(rename = "Ret")]
    Reticolo,
    #[serde(rename = "Scl")]
    Scultore,
    #[serde(rename = "Sco")]
    Scorpione,
    #[serde(rename = "Sct")]
    Scudo,
    #[serde(rename = "Ser")]
    Serpente,
    #[serde(rename = "Sex")]
    Sestante,
    #[serde(rename = "Sge")]
    Freccia,
    #[serde(rename = "Sgr")]
    Sagittario,
    #[serde(rename = "Tau")]
    Toro,
    #[serde(rename = "Tel")]
    Telescopio,
    #[serde(rename = "TrA")]
    TriangoloAustrale,
    #[serde(rename = "Tri")]
    Triangolo,
    #[serde(rename = "Tuc")]
    Tucano,
    #[serde(rename = "UMa")]
    OrsaMaggiore,
    #[serde(rename = "UMi")]
    OrsaMinore,
    #[serde(rename = "Vel")]
    Vele,
    #[serde(rename = "Vir")]
    Vergine,
    #[serde(rename = "Vol")]
    PesceVolante,
    #[serde(rename = "Vul")]
    Volpetta,
}

impl Costellazione {
    pub fn parse(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "AND" => Self::Andromeda,
            "ANT" => Self::MacchinaPneumatica,
            "APS" => Self::UccelloDelParadiso,
            "AQL" => Self::Aquila,
            "AQR" => Self::Aquario,
            "ARA" => Self::Altare,
            "ARI" => Self::Ariete,
            "AUR" => Self::Auriga,
            "BOO" => Self::Boote,
            "CAE" => Self::Bulino,
            "CAM" => Self::Giraffa,
            "CAP" => Self::Capricorno,
            "CAR" => Self::Carena,
            "CAS" => Self::Cassiopea,
            "CEN" => Self::Centauro,
            "CEP" => Self::Cefeo,
            "CET" => Self::Balena,
            "CHA" => Self::Camaleonte,
            "CIR" => Self::Compasso,
            "CMA" => Self::CaneMaggiore,
            "CMI" => Self::CaneMinore,
            "CNC" => Self::Cancro,
            "COL" => Self::Colomba,
            "COM" => Self::ChiomaBerenice,
            "CRA" => Self::CoronaAustrale,
            "CRB" => Self::CoronaBoreale,
            "CRT" => Self::Cratere,
            "CRU" => Self::CroceDelSud,
            "CRV" => Self::Corvo,
            "CVN" => Self::CaniDaCaccia,
            "CYG" => Self::Cigno,
            "DEL" => Self::Delfino,
            "DOR" => Self::Dorado,
            "DRA" => Self::Dragone,
            "EQU" => Self::Cavallino,
            "ERI" => Self::Eridano,
            "FOR" => Self::Fornace,
            "GEM" => Self::Gemelli,
            "GRU" => Self::Gru,
            "HER" => Self::Ercole,
            "HOR" => Self::Orologio,
            "HYA" => Self::IdraFemmina,
            "HYI" => Self::IdraMaschio,
            "IND" => Self::Indiano,
            "LAC" => Self::Lucertola,
            "LEO" => Self::Leone,
            "LEP" => Self::Lepre,
            "LIB" => Self::Bilancia,
            "LMI" => Self::LeoneMinore,
            "LUP" => Self::Lupo,
            "LYN" => Self::Lince,
            "LYR" => Self::Lira,
            "MEN" => Self::Mensa,
            "MIC" => Self::Microscopio,
            "MON" => Self::Unicorno,
            "MUS" => Self::Mosca,
            "NOR" => Self::Regolo,
            "OCT" => Self::Ottante,
            "OPH" => Self::Ofiuco,
            "ORI" => Self::Orione,
            "PAV" => Self::Pavone,
            "PEG" => Self::Pegaso,
            "PER" => Self::Perseo,
            "PHE" => Self::Fenice,
            "PIC" => Self::Pittore,
            "PSA" => Self::PesceAustrale,
            "PSC" => Self::Pesci,
            "PUP" => Self::Poppa,
            "PYX" => Self::Bussola,
            "RET" => Self::Reticolo,
            "SCL" => Self::Scultore,
            "SCO" => Self::Scorpione,
            "SCT" => Self::Scudo,
            "SE1" | "SE2" | "SER" => Self::Serpente,
            "SEX" => Self::Sestante,
            "SGE" => Self::Freccia,
            "SGR" => Self::Sagittario,
            "TAU" => Self::Toro,
            "TEL" => Self::Telescopio,
            "TRA" => Self::TriangoloAustrale,
            "TRI" => Self::Triangolo,
            "TUC" => Self::Tucano,
            "UMA" => Self::OrsaMaggiore,
            "UMI" => Self::OrsaMinore,
            "VEL" => Self::Vele,
            "VIR" => Self::Vergine,
            "VOL" => Self::PesceVolante,
            "VUL" => Self::Volpetta,
            _ => Self::Sconosciuta,
        }
    }
}

impl Default for Costellazione {
    fn default() -> Self {
        Self::Sconosciuta
    }
}

impl fmt::Display for Costellazione {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sconosciuta => write!(f, "Sconosciuta"),
            Self::Andromeda => write!(f, "And"),
            Self::MacchinaPneumatica => write!(f, "Ant"),
            Self::UccelloDelParadiso => write!(f, "Aps"),
            Self::Aquila => write!(f, "Aql"),
            Self::Aquario => write!(f, "Aqr"),
            Self::Altare => write!(f, "Ara"),
            Self::Ariete => write!(f, "Ari"),
            Self::Auriga => write!(f, "Aur"),
            Self::Boote => write!(f, "Boo"),
            Self::Bulino => write!(f, "Cae"),
            Self::Giraffa => write!(f, "Cam"),
            Self::Capricorno => write!(f, "Cap"),
            Self::Carena => write!(f, "Car"),
            Self::Cassiopea => write!(f, "Cas"),
            Self::Centauro => write!(f, "Cen"),
            Self::Cefeo => write!(f, "Cep"),
            Self::Balena => write!(f, "Cet"),
            Self::Camaleonte => write!(f, "Cha"),
            Self::Compasso => write!(f, "Cir"),
            Self::CaneMaggiore => write!(f, "CMa"),
            Self::CaneMinore => write!(f, "CMi"),
            Self::Cancro => write!(f, "Cnc"),
            Self::Colomba => write!(f, "Col"),
            Self::ChiomaBerenice => write!(f, "Com"),
            Self::CoronaAustrale => write!(f, "CrA"),
            Self::CoronaBoreale => write!(f, "CrB"),
            Self::Cratere => write!(f, "Crt"),
            Self::CroceDelSud => write!(f, "Cru"),
            Self::Corvo => write!(f, "Crv"),
            Self::CaniDaCaccia => write!(f, "CVn"),
            Self::Cigno => write!(f, "Cyg"),
            Self::Delfino => write!(f, "Del"),
            Self::Dorado => write!(f, "Dor"),
            Self::Dragone => write!(f, "Dra"),
            Self::Cavallino => write!(f, "Equ"),
            Self::Eridano => write!(f, "Eri"),
            Self::Fornace => write!(f, "For"),
            Self::Gemelli => write!(f, "Gem"),
            Self::Gru => write!(f, "Gru"),
            Self::Ercole => write!(f, "Her"),
            Self::Orologio => write!(f, "Hor"),
            Self::IdraFemmina => write!(f, "Hya"),
            Self::IdraMaschio => write!(f, "Hyi"),
            Self::Indiano => write!(f, "Ind"),
            Self::Lucertola => write!(f, "Lac"),
            Self::Leone => write!(f, "Leo"),
            Self::Lepre => write!(f, "Lep"),
            Self::Bilancia => write!(f, "Lib"),
            Self::LeoneMinore => write!(f, "LMi"),
            Self::Lupo => write!(f, "Lup"),
            Self::Lince => write!(f, "Lyn"),
            Self::Lira => write!(f, "Lyr"),
            Self::Mensa => write!(f, "Men"),
            Self::Microscopio => write!(f, "Mic"),
            Self::Unicorno => write!(f, "Mon"),
            Self::Mosca => write!(f, "Mus"),
            Self::Regolo => write!(f, "Nor"),
            Self::Ottante => write!(f, "Oct"),
            Self::Ofiuco => write!(f, "Oph"),
            Self::Orione => write!(f, "Ori"),
            Self::Pavone => write!(f, "Pav"),
            Self::Pegaso => write!(f, "Peg"),
            Self::Perseo => write!(f, "Per"),
            Self::Fenice => write!(f, "Phe"),
            Self::Pittore => write!(f, "Pic"),
            Self::PesceAustrale => write!(f, "PsA"),
            Self::Pesci => write!(f, "Psc"),
            Self::Poppa => write!(f, "Pup"),
            Self::Bussola => write!(f, "Pyx"),
            Self::Reticolo => write!(f, "Ret"),
            Self::Scultore => write!(f, "Scl"),
            Self::Scorpione => write!(f, "Sco"),
            Self::Scudo => write!(f, "Sct"),
            Self::Serpente => write!(f, "Ser"),
            Self::Sestante => write!(f, "Sex"),
            Self::Freccia => write!(f, "Sge"),
            Self::Sagittario => write!(f, "Sgr"),
            Self::Toro => write!(f, "Tau"),
            Self::Telescopio => write!(f, "Tel"),
            Self::TriangoloAustrale => write!(f, "TrA"),
            Self::Triangolo => write!(f, "Tri"),
            Self::Tucano => write!(f, "Tuc"),
            Self::OrsaMaggiore => write!(f, "UMa"),
            Self::OrsaMinore => write!(f, "UMi"),
            Self::Vele => write!(f, "Vel"),
            Self::Vergine => write!(f, "Vir"),
            Self::PesceVolante => write!(f, "Vol"),
            Self::Volpetta => write!(f, "Vul"),
        }
    }
}

impl<'de> Deserialize<'de> for Costellazione {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::parse(&s))
    }
}

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
    pub id: Option<String>,
    pub tipo: TipoOggetto,
    pub nome_comune: String,
    pub abbr_costellazione: Costellazione,
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
    pub abbr_costellazione: Costellazione,
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
    pub abbr_costellazione: Costellazione,
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
