use crate::import_catalogs::models::*;
use crate::import_catalogs::merge::merge_objects;
use crate::import_catalogs::ngc_download::download_ngc;

/// Generate the complete catalog by loading all embedded data + NGC download,
/// merging with cross-correlation.
pub async fn generate_all(
    skip_ngc: bool,
    no_messier: bool,
    no_caldwell: bool,
    no_arp: bool,
    no_hickson: bool,
    no_abell: bool,
    no_doublestar: bool,
    merge_radius: f64,
) -> Vec<ImportCatalogEntry> {
    let mut all: Vec<ImportCatalogEntry> = Vec::new();

    // 1. NGC
    log::info!("[1/7] Downloading NGC catalog...");
    let ngc = download_ngc(skip_ngc).await;
    log::info!("  NGC/IC: {} objects", ngc.len());
    all.extend(ngc);

    // 2. Messier
    if !no_messier {
        log::info!("[2/7] Loading Messier catalog...");
        let m = crate::import_catalogs::data_messier::load_messier();
        log::info!("  Messier: {} objects", m.len());
        all.extend(m);
    }

    // 3. Caldwell
    if !no_caldwell {
        log::info!("[3/7] Loading Caldwell catalog...");
        let c = crate::import_catalogs::data_caldwell::load_caldwell();
        log::info!("  Caldwell: {} objects", c.len());
        all.extend(c);
    }

    // 4. Arp
    if !no_arp {
        log::info!("[4/7] Loading Arp catalog...");
        let a = crate::import_catalogs::data_arp::load_arp();
        log::info!("  Arp: {} objects", a.len());
        all.extend(a);
    }

    // 5. Hickson
    if !no_hickson {
        log::info!("[5/7] Loading Hickson catalog...");
        let (individuals, groups) = crate::import_catalogs::data_hickson::load_hickson();
        log::info!("  Hickson: {} individuals, {} groups", individuals.len(), groups.len());
        all.extend(individuals);
        all.extend(groups);
    }

    // 6. Abell
    if !no_abell {
        log::info!("[6/7] Loading Abell catalog...");
        let ab = crate::import_catalogs::data_abell::load_abell();
        log::info!("  Abell: {} clusters", ab.len());
        all.extend(ab);
    }

    // 7. Double Stars
    if !no_doublestar {
        log::info!("[7/7] Loading Double Star catalog...");
        let ds = crate::import_catalogs::data_doublestar::load_doublestar();
        log::info!("  Double Stars: {} objects", ds.len());
        all.extend(ds);
    }

    log::info!("Total raw objects: {}", all.len());
    let merged = merge_objects(all, merge_radius);

    // Stats
    let single = merged.iter().filter(|o| !o.multi).count();
    let multi = merged.iter().filter(|o| o.multi).count();
    let with_ngc = merged.iter().filter(|o| o.cataloghi.iter().any(|c| c.catalog_id == "NGC" || c.catalog_id == "IC")).count();
    let with_m = merged.iter().filter(|o| o.cataloghi.iter().any(|c| c.catalog_id == "M")).count();
    let with_c = merged.iter().filter(|o| o.cataloghi.iter().any(|c| c.catalog_id == "C")).count();
    let with_arp = merged.iter().filter(|o| o.cataloghi.iter().any(|c| c.catalog_id == "Arp")).count();
    let with_hcg = merged.iter().filter(|o| o.cataloghi.iter().any(|c| c.catalog_id == "HCG")).count();
    let with_abell = merged.iter().filter(|o| o.cataloghi.iter().any(|c| c.catalog_id == "Abell")).count();
    let with_ds = merged.iter().filter(|o| o.cataloghi.iter().any(|c| c.catalog_id == "DS")).count();
    let cross = merged.iter().filter(|o| {
        let ids: std::collections::HashSet<_> = o.cataloghi.iter().filter(|c| ["NGC","IC","M","C","Arp","HCG","Abell","DS"].contains(&c.catalog_id.as_str())).map(|c| &c.catalog_id).collect();
        ids.len() >= 2
    }).count();

    log::info!("Merge complete: {} objects ({} single, {} multi)", merged.len(), single, multi);
    log::info!("  NGC/IC: {with_ngc}, M: {with_m}, C: {with_c}, Arp: {with_arp}, HCG: {with_hcg}, Abell: {with_abell}, DS: {with_ds}");
    log::info!("  Cross-catalog matches: {cross}");

    merged
}
