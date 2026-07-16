use crate::statics::models::EnumValue;

const TIPO_STRUMENTAZIONE: &[EnumValue] = &[
    EnumValue { name: "Telescopio", value: "telescopio" },
    EnumValue { name: "Barlow/Correttore", value: "barlow_correttore" },
    EnumValue { name: "Torretta", value: "torretta" },
    EnumValue { name: "Binocolo", value: "binocolo" },
    EnumValue { name: "Oculare", value: "oculare" },
    EnumValue { name: "Cercatore", value: "cercatore" },
    EnumValue { name: "Altro", value: "altro" },
];

const TIPO_OGGETTO: &[EnumValue] = &[
    EnumValue { name: "Galassia", value: "GAL" },
    EnumValue { name: "Galassia Ellittica", value: "GAL_EL" },
    EnumValue { name: "Galassia Lenticolare", value: "GAL_LN" },
    EnumValue { name: "Galassia Spirale", value: "GAL_SP" },
    EnumValue { name: "Galassia Barrata", value: "GAL_SB" },
    EnumValue { name: "Galassia Irregolare", value: "GAL_IR" },
    EnumValue { name: "Galassia Nana", value: "GAL_DW" },
    EnumValue { name: "Galassia Peculiare", value: "GAL_PEC" },
    EnumValue { name: "Galassia Attiva", value: "GAL_AGN" },
    EnumValue { name: "Ammasso Aperto", value: "OpC" },
    EnumValue { name: "Ammasso Globulare", value: "GCl" },
    EnumValue { name: "Nebulosa", value: "Neb" },
    EnumValue { name: "Nebulosa a Emissione", value: "EmN" },
    EnumValue { name: "Nebulosa a Riflessione", value: "RfN" },
    EnumValue { name: "Nebulosa Planetaria", value: "PN" },
    EnumValue { name: "Resto di Supernova", value: "SNR" },
    EnumValue { name: "Regione HII", value: "HII" },
    EnumValue { name: "Nebulosa Oscura", value: "DkNeb" },
    EnumValue { name: "Ammasso di Galassie", value: "GCL" },
    EnumValue { name: "Gruppo di Galassie", value: "HCG" },
    EnumValue { name: "Stella", value: "Star" },
    EnumValue { name: "Stella Doppia", value: "2Star" },
    EnumValue { name: "Asterismo", value: "Aster" },
    EnumValue { name: "Nube Stellare", value: "StarCloud" },
    EnumValue { name: "Nebulosa + Ammasso", value: "Neb+OpC" },
    EnumValue { name: "Quasar", value: "QSO" },
    EnumValue { name: "Pulsar", value: "PSR" },
];

const COSTELLAZIONI: &[EnumValue] = &[
    EnumValue { name: "Sconosciuta", value: "Sconosciuta" },
    EnumValue { name: "Andromeda", value: "And" },
    EnumValue { name: "Macchina Pneumatica", value: "Ant" },
    EnumValue { name: "Uccello del Paradiso", value: "Aps" },
    EnumValue { name: "Aquila", value: "Aql" },
    EnumValue { name: "Acquario", value: "Aqr" },
    EnumValue { name: "Altare", value: "Ara" },
    EnumValue { name: "Ariete", value: "Ari" },
    EnumValue { name: "Auriga", value: "Aur" },
    EnumValue { name: "Boote", value: "Boo" },
    EnumValue { name: "Bulino", value: "Cae" },
    EnumValue { name: "Giraffa", value: "Cam" },
    EnumValue { name: "Capricorno", value: "Cap" },
    EnumValue { name: "Carena", value: "Car" },
    EnumValue { name: "Cassiopea", value: "Cas" },
    EnumValue { name: "Centauro", value: "Cen" },
    EnumValue { name: "Cefeo", value: "Cep" },
    EnumValue { name: "Balena", value: "Cet" },
    EnumValue { name: "Camaleonte", value: "Cha" },
    EnumValue { name: "Compasso", value: "Cir" },
    EnumValue { name: "Cane Maggiore", value: "CMa" },
    EnumValue { name: "Cane Minore", value: "CMi" },
    EnumValue { name: "Cancro", value: "Cnc" },
    EnumValue { name: "Colomba", value: "Col" },
    EnumValue { name: "Chioma di Berenice", value: "Com" },
    EnumValue { name: "Corona Australe", value: "CrA" },
    EnumValue { name: "Corona Boreale", value: "CrB" },
    EnumValue { name: "Cratere", value: "Crt" },
    EnumValue { name: "Croce del Sud", value: "Cru" },
    EnumValue { name: "Corvo", value: "Crv" },
    EnumValue { name: "Cani da Caccia", value: "CVn" },
    EnumValue { name: "Cigno", value: "Cyg" },
    EnumValue { name: "Delfino", value: "Del" },
    EnumValue { name: "Dorado", value: "Dor" },
    EnumValue { name: "Dragone", value: "Dra" },
    EnumValue { name: "Cavallino", value: "Equ" },
    EnumValue { name: "Eridano", value: "Eri" },
    EnumValue { name: "Fornace", value: "For" },
    EnumValue { name: "Gemelli", value: "Gem" },
    EnumValue { name: "Gru", value: "Gru" },
    EnumValue { name: "Ercole", value: "Her" },
    EnumValue { name: "Orologio", value: "Hor" },
    EnumValue { name: "Idra Femmina", value: "Hya" },
    EnumValue { name: "Idra Maschio", value: "Hyi" },
    EnumValue { name: "Indiano", value: "Ind" },
    EnumValue { name: "Lucertola", value: "Lac" },
    EnumValue { name: "Leone", value: "Leo" },
    EnumValue { name: "Lepre", value: "Lep" },
    EnumValue { name: "Bilancia", value: "Lib" },
    EnumValue { name: "Leone Minore", value: "LMi" },
    EnumValue { name: "Lupo", value: "Lup" },
    EnumValue { name: "Lince", value: "Lyn" },
    EnumValue { name: "Lira", value: "Lyr" },
    EnumValue { name: "Mensa", value: "Men" },
    EnumValue { name: "Microscopio", value: "Mic" },
    EnumValue { name: "Unicorno", value: "Mon" },
    EnumValue { name: "Mosca", value: "Mus" },
    EnumValue { name: "Regolo", value: "Nor" },
    EnumValue { name: "Ottante", value: "Oct" },
    EnumValue { name: "Ofiuco", value: "Oph" },
    EnumValue { name: "Orione", value: "Ori" },
    EnumValue { name: "Pavone", value: "Pav" },
    EnumValue { name: "Pegaso", value: "Peg" },
    EnumValue { name: "Perseo", value: "Per" },
    EnumValue { name: "Fenice", value: "Phe" },
    EnumValue { name: "Pittore", value: "Pic" },
    EnumValue { name: "Pesce Australe", value: "PsA" },
    EnumValue { name: "Pesci", value: "Psc" },
    EnumValue { name: "Poppa", value: "Pup" },
    EnumValue { name: "Bussola", value: "Pyx" },
    EnumValue { name: "Reticolo", value: "Ret" },
    EnumValue { name: "Scultore", value: "Scl" },
    EnumValue { name: "Scorpione", value: "Sco" },
    EnumValue { name: "Scudo", value: "Sct" },
    EnumValue { name: "Serpente", value: "Ser" },
    EnumValue { name: "Sestante", value: "Sex" },
    EnumValue { name: "Freccia", value: "Sge" },
    EnumValue { name: "Sagittario", value: "Sgr" },
    EnumValue { name: "Toro", value: "Tau" },
    EnumValue { name: "Telescopio", value: "Tel" },
    EnumValue { name: "Triangolo Australe", value: "TrA" },
    EnumValue { name: "Triangolo", value: "Tri" },
    EnumValue { name: "Tucano", value: "Tuc" },
    EnumValue { name: "Orsa Maggiore", value: "UMa" },
    EnumValue { name: "Orsa Minore", value: "UMi" },
    EnumValue { name: "Vele", value: "Vel" },
    EnumValue { name: "Vergine", value: "Vir" },
    EnumValue { name: "Pesce Volante", value: "Vol" },
    EnumValue { name: "Volpetta", value: "Vul" },
];

pub struct StaticsRepository;

impl StaticsRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn get_tipi_strumentazione(&self) -> &'static [EnumValue] {
        TIPO_STRUMENTAZIONE
    }

    pub fn get_tipi_oggetto(&self) -> &'static [EnumValue] {
        TIPO_OGGETTO
    }

    pub fn get_costellazioni(&self) -> &'static [EnumValue] {
        COSTELLAZIONI
    }
}
