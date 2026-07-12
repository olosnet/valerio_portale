use crate::import_catalogs::coords::angular_separation_arcsec;
use crate::import_catalogs::models::*;
use serde::Serialize;

/// Converte coord_ar compatto (HHMMSS) in gradi decimali.
fn parse_coord_ar(ra: &str) -> Option<f64> {
    if ra.len() < 6 { return None; }
    let h: f64 = ra[0..2].parse().ok()?;
    let m: f64 = ra[2..4].parse().ok()?;
    let s: f64 = ra[4..6].parse().ok()?;
    let frac: f64 = if ra.len() > 7 && ra.as_bytes()[6] == b'.' {
        ra[7..].parse::<f64>().unwrap_or(0.0) / 10f64.powi(ra[7..].len() as i32)
    } else { 0.0 };
    Some(15.0 * (h + m / 60.0 + (s + frac) / 3600.0))
}

/// Converte coord_dec compatto (+DDMMSS) in gradi decimali.
fn parse_coord_dec(dec: &str) -> Option<f64> {
    if dec.len() < 7 { return None; }
    let sign = if dec.as_bytes()[0] == b'-' { -1.0 } else { 1.0 };
    let d: f64 = dec[1..3].parse().ok()?;
    let m: f64 = dec[3..5].parse().ok()?;
    let s: f64 = dec[5..7].parse().ok()?;
    Some(sign * (d + m / 60.0 + s / 3600.0))
}

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub index: usize,
    pub catalogs_summary: String,
    pub name: String,
    pub status: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub async fn validate_against_simbad(
    objects: &[ImportCatalogEntry],
    max: usize,
    tolerance_arcsec: f64,
) -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let limit = max.min(objects.len());

    for idx in 0..limit {
        let obj = &objects[idx];

        let catalogs_summary = obj
            .cataloghi
            .iter()
            .map(|c| format!("{} {}", c.catalog_id, c.catalog_nr))
            .collect::<Vec<_>>()
            .join(", ");

        let query_name = obj
            .cataloghi
            .iter()
            .find(|c| c.catalog_id == "NGC")
            .or_else(|| obj.cataloghi.iter().find(|c| c.catalog_id == "IC"))
            .or_else(|| obj.cataloghi.iter().find(|c| c.catalog_id == "M"))
            .or_else(|| obj.cataloghi.iter().find(|c| c.catalog_id == "C"))
            .or_else(|| obj.cataloghi.iter().find(|c| c.catalog_id == "Arp"))
            .or_else(|| obj.cataloghi.iter().find(|c| c.catalog_id == "HCG"))
            .or_else(|| obj.cataloghi.iter().find(|c| c.catalog_id == "Abell"))
            .or_else(|| obj.cataloghi.first())
            .map(|c| {
                // Mappa alcuni cataloghi al nome SIMBAD corretto
                match c.catalog_id.as_str() {
                    "C" => format!("Caldwell {}", c.catalog_nr),
                    "HCG" => format!("HCG {}", c.catalog_nr.split('-').next().unwrap_or(&c.catalog_nr)),
                    _ => format!("{} {}", c.catalog_id, c.catalog_nr),
                }
            })
            .unwrap_or_else(|| String::from("sconosciuto"));

        log::info!("Validazione [{}/{}] {}...", idx + 1, limit, query_name);

        // Breve pausa tra le richieste per non sovraccaricare il servizio
        if idx > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        }

        let url = format!(
            "https://cds.unistra.fr/cgi-bin/nph-sesame/-oIx?{}",
            url_encode(&query_name)
        );

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        match query_simbad(&url, &query_name).await {
            Ok((simbad_ra, simbad_dec, aliases)) => {
                // Controllo cross-referenze catalogo
                let alias_normalized: Vec<String> = aliases.iter()
                    .map(|a| a.to_uppercase().replace("  ", " ").trim().to_string())
                    .collect();

                for cat in &obj.cataloghi {
                    let cat_nr_clean = cat.catalog_nr.trim_start_matches('0');
                    let simbad_name = match cat.catalog_id.as_str() {
                        "C" => format!("Caldwell {}", cat_nr_clean),
                        _ => format!("{} {}", cat.catalog_id, cat_nr_clean),
                    };
                    let simbad_name_upper = simbad_name.to_uppercase();

                    if !alias_normalized.iter().any(|a| a.contains(&simbad_name_upper)) {
                        // Per HCG con lettera (es. "73-a") controlla anche senza lettera ("HCG 73")
                        if cat.catalog_id == "HCG" && cat.catalog_nr.contains('-') {
                            let base = format!("HCG {}", cat.catalog_nr.split('-').next().unwrap());
                            if alias_normalized.iter().any(|a| a.contains(&base.to_uppercase())) {
                                continue;
                            }
                        }
                        warnings.push(format!("Cross-ref {} non confermata da SIMBAD", simbad_name));
                    }
                }

                // Controllo coordinate
                let our_ra = obj.ra_decimal.or_else(|| parse_coord_ar(&obj.coord_ar));
                let our_dec = obj.dec_decimal.or_else(|| parse_coord_dec(&obj.coord_dec));

                match (our_ra, our_dec) {
                    (Some(ra), Some(dec)) => {
                        let sep = angular_separation_arcsec(ra, dec, simbad_ra, simbad_dec);
                        if sep > tolerance_arcsec {
                            errors.push(format!(
                                "Distanza SIMBAD: {:.1}\" > tolleranza {:.1}\"",
                                sep, tolerance_arcsec
                            ));
                        } else if sep > 5.0 {
                            warnings.push(format!("Distanza SIMBAD: {:.1}\"", sep));
                        }
                    }
                    (_, _) => {
                        warnings.push("Coordinate decimali mancanti nell'oggetto".to_string());
                    }
                }
            }
            Err(e) => {
                warnings.push(format!("SIMBAD query fallita: {e}"));
            }
        }

        let status = if !errors.is_empty() {
            "ERROR"
        } else if !warnings.is_empty() {
            "WARN"
        } else {
            "OK"
        };

        results.push(ValidationResult {
            index: idx,
            catalogs_summary,
            name: query_name,
            status: status.to_string(),
            errors,
            warnings,
        });
    }

    results
}

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

async fn query_simbad(url: &str, query_name: &str) -> Result<(f64, f64, Vec<String>), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "app_managment/0.1.0")
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("Read error: {e}"))?;

    if !status.is_success() {
        return Err(format!("HTTP {status}: {text}"));
    }

    log::debug!("Sesame response for {} ({} bytes):\n{}", query_name, text.len(), &text[..text.len().min(500)]);

    let resolver_start = text
        .find("<Resolver ") // tag con attributi: <Resolver name="...">
        .or_else(|| text.find("<Resolver>"))
        .ok_or_else(|| {
            log::warn!("<Resolver> non trovato. text.len={}, contains Resolver={}",
                text.len(), text.contains("Resolver"));
            format!("Nessun risultato SIMBAD per {query_name}")
        })?;
    let resolver_end = text[resolver_start..]
        .find("</Resolver>")
        .ok_or_else(|| "XML malformato: </Resolver> non trovato".to_string())?;
    let resolver_text = &text[resolver_start..resolver_start + resolver_end + "</Resolver>".len()];

    if resolver_text.contains("<INFO>Nothing found</INFO>") {
        return Err(format!("Nessun risultato SIMBAD per {query_name}"));
    }

    // Estrai coordinate in gradi decimali da <jradeg> e <jdedeg>
    let ra = extract_decimal_tag(resolver_text, "jradeg")
        .ok_or_else(|| format!("jradeg non trovato per {query_name}"))?;
    let dec = extract_decimal_tag(resolver_text, "jdedeg")
        .ok_or_else(|| format!("jdedeg non trovato per {query_name}"))?;

    let mut aliases = extract_elements(resolver_text, "oname");
    aliases.extend(extract_elements(resolver_text, "alias"));
    let aliases: Vec<String> = aliases.into_iter().map(|s| s.trim().to_string()).collect();

    Ok((ra, dec, aliases))
}

/// Estrae il valore numerico da un tag XML come <tag>VAL</tag>.
fn extract_decimal_tag(xml: &str, tag: &str) -> Option<f64> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let after = start + open.len();
    let end = xml[after..].find(&close)?;
    xml[after..after + end].trim().parse::<f64>().ok()
}

fn extract_elements(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut results = Vec::new();
    let mut search_from = 0;
    while let Some(start) = xml[search_from..].find(&open) {
        let abs_start = search_from + start + open.len();
        if let Some(end) = xml[abs_start..].find(&close) {
            results.push(xml[abs_start..abs_start + end].to_string());
            search_from = abs_start + end + close.len();
        } else {
            break;
        }
    }
    results
}
