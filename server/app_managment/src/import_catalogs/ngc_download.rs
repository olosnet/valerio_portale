use crate::import_catalogs::coords::*;
use crate::import_catalogs::models::*;

const LOCAL_NGC_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/import_catalogs/NGC.csv");
const REMOTE_NGC_URL: &str = "https://raw.githubusercontent.com/mattiaverga/OpenNGC/refs/heads/master/database_files/NGC.csv";

/// Legge il contenuto NGC: prima tenta il file locale, poi lo scarica da remoto.
fn load_ngc_text() -> Option<String> {
    // Prova file locale
    if let Ok(text) = std::fs::read_to_string(LOCAL_NGC_PATH) {
        log::info!("NGC caricato da file locale: {}", LOCAL_NGC_PATH);
        return Some(text);
    }

    // Scarica da remoto
    log::info!("NGC non trovato in locale, scarico da {}", REMOTE_NGC_URL);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(REMOTE_NGC_URL)
        .header("User-Agent", "app_managment/0.1.0")
        .send()
        .ok()?;

    let text = resp.text().ok()?;

    // Salva in locale per usi futuri
    if let Err(e) = std::fs::write(LOCAL_NGC_PATH, &text) {
        log::warn!("Impossibile salvare NGC in locale: {e}");
    } else {
        log::info!("NGC salvato in {}", LOCAL_NGC_PATH);
    }

    Some(text)
}

/// Parse RA from HMS format (e.g. "00:08:27.05") to decimal degrees.
fn parse_hms_to_decimal(hms: &str) -> Option<f64> {
    let parts: Vec<&str> = hms.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let s: f64 = parts[2].parse().ok()?;
    Some(ra_hms_to_decimal(h, m, s))
}

/// Parse Dec from DMS format (e.g. "+27:43:03.6") to decimal degrees.
fn parse_dms_to_decimal(dms: &str) -> Option<f64> {
    let dms = dms.trim();
    if dms.is_empty() {
        return None;
    }
    let sign = dms.chars().next()?;
    let body = &dms[1..];
    let parts: Vec<&str> = body.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let d: f64 = parts[0].parse().ok()?;
    let m: f64 = parts[1].parse().ok()?;
    let s: f64 = parts[2].parse().ok()?;
    Some(dec_dms_to_decimal(sign, d, m, s))
}

/// Map OpenNGC type codes to our tipo field.
fn map_ngc_type(code: &str) -> &str {
    match code.trim() {
        "G" | "GPair" | "GTrpl" | "GGroup" | "GClust" => "GAL",
        "OC" | "OCl" | "OpC" => "OpC",
        "GC" | "GCl" | "Gb" => "GCl",
        "PN" | "PlN" | "PlN?" => "PN",
        "Nb" | "Neb" => "Neb",
        "EmN" => "EmN",
        "RfN" => "RfN",
        "SNR" => "SNR",
        "HII" => "HII",
        "DrkN" | "DkNeb" => "DkNeb",
        "**" | "*" => "Star",
        "2Star" | "3Star" | "4Star" | "5Star" | "6Star" | "8Star" | "9Star" | "Association*" => "2Star",
        "Ast" | "Aster" => "Aster",
        "?" | "None" => "GAL",
        _ => "GAL",
    }
}

pub async fn download_ngc(skip: bool) -> Vec<ImportCatalogEntry> {
    if skip {
        return Vec::new();
    }

    let text = match load_ngc_text() {
        Some(t) => t,
        None => {
            log::error!("Impossibile caricare il catalogo NGC");
            return Vec::new();
        }
    };

    let mut lines = text.lines();

    // Header is first line, semicolon-delimited
    let header_line = match lines.next() {
        Some(line) if !line.starts_with('#') => line,
        Some(_) => {
            // Skip comment lines
            loop {
                match lines.next() {
                    Some(line) if !line.starts_with('#') => break line,
                    None => return Vec::new(),
                    _ => continue,
                }
            }
        }
        None => return Vec::new(),
    };

    let headers: Vec<&str> = header_line
        .split(';')
        .map(|h| h.trim().trim_matches('"'))
        .collect();

    log::info!("OpenNGC headers: {:?}", headers);

    // Find column indices by exact name
    let col_name = headers.iter().position(|h| *h == "Name");
    let col_type = headers.iter().position(|h| *h == "Type");
    let col_ra = headers.iter().position(|h| *h == "RA");
    let col_dec = headers.iter().position(|h| *h == "Dec");
    let col_majax = headers.iter().position(|h| *h == "MajAx");
    let col_minax = headers.iter().position(|h| *h == "MinAx");
    let col_vmag = headers.iter().position(|h| *h == "V-Mag");
    let col_const = headers.iter().position(|h| *h == "Const");

    let (Some(col_name), Some(col_ra), Some(col_dec)) = (col_name, col_ra, col_dec) else {
        log::error!("Colonne Name/RA/Dec non trovate nell'header OpenNGC");
        return Vec::new();
    };

    let max_col = col_name
        .max(col_ra)
        .max(col_dec)
        .max(col_type.unwrap_or(0))
        .max(col_majax.unwrap_or(0))
        .max(col_minax.unwrap_or(0))
        .max(col_vmag.unwrap_or(0))
        .max(col_const.unwrap_or(0));

    let mut results = Vec::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(';').map(|f| f.trim().trim_matches('"')).collect();
        if fields.len() <= max_col {
            continue;
        }

        // Extract name and catalog prefix
        let name = fields[col_name];
        let (catalog_id, catalog_nr) = if let Some(stripped) = name.strip_prefix("IC") {
            ("IC", stripped.to_string())
        } else if let Some(stripped) = name.strip_prefix("NGC") {
            ("NGC", stripped.to_string())
        } else {
            continue; // Skip non-NGC/IC entries
        };

        // RA and Dec in HMS/DMS format
        let ra_str = fields[col_ra];
        let dec_str = fields[col_dec];

        let ra_deg = match parse_hms_to_decimal(ra_str) {
            Some(v) => v,
            None => continue,
        };
        let dec_deg = match parse_dms_to_decimal(dec_str) {
            Some(v) => v,
            None => continue,
        };

        // Type mapping
        let raw_type = col_type.map(|ci| fields[ci]).unwrap_or("");
        let tipo = map_ngc_type(raw_type).to_string();

        // Constellation
        let constellation = col_const
            .and_then(|ci| {
                let v = fields[ci];
                if v.is_empty() { None } else { Some(v.to_string()) }
            })
            .unwrap_or_default();

        // Magnitude
        let mag = col_vmag
            .and_then(|ci| {
                let v = fields[ci];
                if v.is_empty() { None } else { v.parse::<f64>().ok() }
            });

        // Dimensions (MajAx/MinAx in arcminutes, convert to arcseconds)
        let dim_a_arcmin: Option<f64> = col_majax.and_then(|ci| {
            let v = fields[ci];
            if v.is_empty() { None } else { v.parse::<f64>().ok() }
        });
        let dim_b_arcmin: Option<f64> = col_minax.and_then(|ci| {
            let v = fields[ci];
            if v.is_empty() { None } else { v.parse::<f64>().ok() }
        });

        let dim_apparenti = if dim_a_arcmin.is_some() || dim_b_arcmin.is_some() {
            Some(ImportDimApp {
                secs_a: dim_a_arcmin.map(|a| (a * 60.0).round() as i32),
                secs_b: dim_b_arcmin.map(|b| (b * 60.0).round() as i32),
            })
        } else {
            None
        };

        let ra_coord = decimal_ra_to_coord(ra_deg);
        let dec_coord = decimal_dec_to_coord(dec_deg);

        results.push(ImportCatalogEntry {
            cataloghi: vec![CatalogEntry {
                catalog_id: catalog_id.to_string(),
                catalog_nr,
            }],
            tipo,
            nome_comune: String::new(),
            abbr_costellazione: constellation,
            coord_ar: ra_coord,
            coord_dec: dec_coord,
            mag_apparente: mag,
            dim_apparenti,
            note: String::new(),
            multi: false,
            ra_decimal: Some(ra_deg),
            dec_decimal: Some(dec_deg),
        });
    }

    results
}
