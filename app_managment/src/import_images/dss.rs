use serde::{Deserialize, Serialize};

/// DSS image entry for an astronomical object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DssImageEntry {
    pub index: usize,
    pub catalogs: Vec<CatalogRef>,
    pub ra: String,
    pub dec: String,
    pub url: String,
    pub survey: String,
    pub fov_arcmin: f64,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogRef {
    pub catalog_id: String,
    pub catalog_nr: String,
}

/// Minimum FOV in arcminutes.
const MIN_FOV_ARCMIN: f64 = 6.0;
/// Maximum FOV in arcminutes.
const MAX_FOV_ARCMIN: f64 = 60.0;
/// Default FOV when no dimension info is available.
const DEFAULT_FOV_ARCMIN: f64 = 15.0;
/// Multiplier: FOV = max_dimension_arcmin * PADDING_FACTOR to include the object with padding.
const PADDING_FACTOR: f64 = 3.0;

/// Compute optimal FOV based on apparent dimensions.
/// Returns FOV in arcminutes that encompasses the object with padding.
pub fn compute_fov(secs_a: Option<i32>, secs_b: Option<i32>) -> f64 {
    let dim_a_arcmin = secs_a.map(|a| a as f64 / 60.0).unwrap_or(0.0);
    let dim_b_arcmin = secs_b.map(|b| b as f64 / 60.0).unwrap_or(0.0);
    let max_dim = dim_a_arcmin.max(dim_b_arcmin);

    if max_dim <= 0.0 {
        return DEFAULT_FOV_ARCMIN;
    }

    (max_dim * PADDING_FACTOR).max(MIN_FOV_ARCMIN).min(MAX_FOV_ARCMIN).round()
}

/// Build a DSS image URL from compact coordinate strings.
pub fn build_dss_url(
    ra_str: &str,
    dec_str: &str,
    survey: &str,
    fov_arcmin: f64,
) -> String {
    let (ra_h, ra_m, ra_s) = parse_ra_components(ra_str);
    let (dec_sign, dec_d, dec_m, dec_s) = parse_dec_components(dec_str);

    format!(
        "https://archive.stsci.edu/cgi-bin/dss_search?v={survey}&r={ra_h}+{ra_m}+{ra_s}&d={dec_sign}{dec_d}+{dec_m}+{dec_s}&e=J2000&h={fov}&w={fov}&f=gif&c=none&fov=NONE&v3=",
        survey = survey,
        ra_h = ra_h,
        ra_m = ra_m,
        ra_s = ra_s,
        dec_sign = dec_sign,
        dec_d = dec_d,
        dec_m = dec_m,
        dec_s = dec_s,
        fov = fov_arcmin,
    )
}

fn parse_ra_components(ra: &str) -> (&str, &str, &str) {
    let ra = ra.trim();
    if ra.is_empty() {
        return ("0", "0", "0");
    }
    let base = match ra.split_once('.') {
        Some((b, _)) => b,
        None => ra,
    };
    let h = if base.len() >= 2 { &base[0..2] } else { "00" };
    let m = if base.len() >= 4 { &base[2..4] } else { "00" };
    let s = if base.len() >= 6 { &base[4..6] } else { "00" };
    (h, m, s)
}

fn parse_dec_components(dec: &str) -> (char, &str, &str, &str) {
    let dec = dec.trim();
    if dec.is_empty() {
        return ('+', "00", "00", "00");
    }
    let sign = dec.chars().next().unwrap_or('+');
    let body = &dec[1..];
    let d = if body.len() >= 2 { &body[0..2] } else { "00" };
    let m = if body.len() >= 4 { &body[2..4] } else { "00" };
    let s = if body.len() >= 6 { &body[4..6] } else { "00" };
    (sign, d, m, s)
}

/// Generate DSS image entries for all catalog objects.
/// Uses dynamic FOV based on each object's apparent dimensions.
pub fn generate_dss_entries(
    objects: &[crate::import_catalogs::models::ImportCatalogEntry],
    survey: &str,
    _default_fov: f64, // kept for API compatibility, ignored in favor of dynamic FOV
) -> Vec<DssImageEntry> {
    objects
        .iter()
        .enumerate()
        .filter_map(|(i, obj)| {
            if obj.coord_ar.is_empty() || obj.coord_dec.is_empty() {
                return None;
            }

            // Dynamic FOV: use apparent dimensions if available
            let fov = compute_fov(
                obj.dim_apparenti.as_ref().and_then(|d| d.secs_a),
                obj.dim_apparenti.as_ref().and_then(|d| d.secs_b),
            );

            let url = build_dss_url(&obj.coord_ar, &obj.coord_dec, survey, fov);
            let catalogs: Vec<CatalogRef> = obj
                .cataloghi
                .iter()
                .map(|c| CatalogRef {
                    catalog_id: c.catalog_id.clone(),
                    catalog_nr: c.catalog_nr.clone(),
                })
                .collect();
            Some(DssImageEntry {
                index: i,
                catalogs,
                ra: obj.coord_ar.clone(),
                dec: obj.coord_dec.clone(),
                url,
                survey: survey.to_string(),
                fov_arcmin: fov,
                local_path: None,
            })
        })
        .collect()
}

/// Download a DSS image from URL to a local file.
pub async fn download_dss_image(
    url: &str,
    dest_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("dss-downloader/1.0")
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(format!("HTTP {} for {}", response.status(), url).into());
    }

    let bytes = response.bytes().await?;
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest_path, bytes)?;
    Ok(())
}
