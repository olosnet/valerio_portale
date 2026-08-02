mod errors;
mod import_catalogs;
mod import_images;
mod redis_cache;
mod tests;

use app_modules::{
    astronomia::{
        oggetti_astronomici::{
            OggettiAstronomiciModule,
            repos::MongoOggettoAstronomicoModel,
        },
        sessioni_osservative::SessioniOsservativeModule,
        siti_osservativi::SitiOsservativiModule,
        strumentazione::StrumentazioneModule,
    },
    base::{
        enums::EnumsModule, filemanager::FileManagerModule,
        filemanager_images::FileManagerImagesModule, groups::GroupsModule, users::UsersModule,
    },
};
use clap::{Arg, ArgAction, Command};
use cornetti::{
    conf::CornettiConf,
    core::traits::BaseModel,
    filemanager::{
        confs::FileManagerConf, helpers::upload_file_from_path,
        traits::FileManagerRepositoryTrait,
    },
    mongo::{
        confs::MongoDBConfig, helpers::init_mongo_modules, services::MongoDBService,
        traits::{MongoBaseModel, MongoBaseModule},
    },
};
use futures::TryStreamExt;
use bson::doc as bdoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use validator::Validate;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Converte un `CornettiError` in `Box<dyn Error>` per i comandi CLI.
pub(crate) fn conf_error(err: cornetti::core::models::CornettiError) -> Box<dyn std::error::Error> {
    format!("{}: {}", err.detail, err.internal_detail).into()
}

// ---------------------------------------------------------------------------
// Match types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct ImportMatchEntry {
    mongo_id: String,
    cataloghi: Vec<CatalogMatchKey>,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
struct CatalogMatchKey {
    catalog_id: String,
    catalog_nr: String,
}

// ---------------------------------------------------------------------------
// Helper: elimina immagine associata a un oggetto
// ---------------------------------------------------------------------------

/// Cancella un'immagine dal filemanager (file + DB) dato il nome file randomizzato.
/// Usata durante il purge delle importazioni e prima di sovrascrivere immagini.
async fn delete_oggetto_image(
    mongo: &Arc<MongoDBService>,
    image_filename: &str,
    tenant_id: &str,
    app_namespace: &str,
    fm_conf: &FileManagerConf,
) -> Result<(), Box<dyn std::error::Error>> {
    use cornetti::filemanager::helpers::retrieve_file_entry_path;

    let fm_coll = mongo.db().collection::<bson::Document>("filemanager");

    let file = match fm_coll.find_one(bdoc! {
        "filename": image_filename,
        "app_source": app_namespace,
    }).await.map_err(|e| format!("query filemanager: {e}"))? {
        Some(f) => f,
        None => return Ok(()),
    };

    let oid = file.get_object_id("_id").ok();
    let uploader_oid = file.get_object_id("uploader_id").ok();
    let uploader_id = uploader_oid.as_ref().map(|id| id.to_hex());

    // Cancella file fisico
    if let Some(ref uid) = uploader_id {
        let path = retrieve_file_entry_path(
            tenant_id, app_namespace, uid, image_filename, fm_conf,
        ).await;
        if let Ok(p) = path {
            let _ = std::fs::remove_file(&p);
        }
    }

    // Cancella record DB
    if let Some(id) = oid {
        fm_coll.delete_one(bdoc! { "_id": id }).await.map_err(|e| format!("delete file: {e}"))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Register modules
// ---------------------------------------------------------------------------

async fn register_all_modules(mongo: &MongoDBService) -> Result<(), Box<dyn std::error::Error>> {
    init_mongo_modules(mongo).await?;
    UsersModule::register(mongo).await?;
    GroupsModule::register(mongo).await?;
    EnumsModule::register(mongo).await?;
    OggettiAstronomiciModule::register(mongo).await?;
    SitiOsservativiModule::register(mongo).await?;
    SessioniOsservativeModule::register(mongo).await?;
    StrumentazioneModule::register(mongo).await?;
    FileManagerModule::register(mongo).await?;
    FileManagerImagesModule::register(mongo).await?;
    log::info!("All modules registered successfully.");
    Ok(())
}

// ===========================================================================
// catalog generate — genera il JSON del catalogo astronomico
// ===========================================================================

async fn cmd_catalog_generate(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let output = args.get_one::<String>("output").map(|s| s.as_str()).unwrap_or("oggetti_astronomici_import.json");
    let radius = args.get_one::<f64>("merge-radius").copied().unwrap_or(30.0);

    let objects = import_catalogs::generate::generate_all(
        args.get_flag("skip-download"), args.get_flag("no-messier"),
        args.get_flag("no-caldwell"), args.get_flag("no-arp"),
        args.get_flag("no-hickson"), args.get_flag("no-abell"),
        args.get_flag("no-doublestar"), radius,
    ).await;

    let json = serde_json::to_string_pretty(&objects)?;
    std::fs::write(output, json)?;
    println!("Generati {} oggetti -> {}", objects.len(), output);
    Ok(())
}

fn catalog_generate_cmd() -> Command {
    Command::new("generate")
        .about("Genera il file JSON del catalogo astronomico (NGC + Messier + Caldwell + Arp + Hickson + Abell + Double Star)")
        .arg(Arg::new("output").short('o').long("output").value_name("FILE").default_value("oggetti_astronomici_import.json").help("File JSON in output"))
        .arg(Arg::new("skip-download").long("skip-download").action(ArgAction::SetTrue).help("Usa solo il file NGC locale, non scaricare da remoto"))
        .arg(Arg::new("no-messier").long("no-messier").action(ArgAction::SetTrue).help("Escludi catalogo Messier"))
        .arg(Arg::new("no-caldwell").long("no-caldwell").action(ArgAction::SetTrue).help("Escludi catalogo Caldwell"))
        .arg(Arg::new("no-arp").long("no-arp").action(ArgAction::SetTrue).help("Escludi catalogo Arp"))
        .arg(Arg::new("no-hickson").long("no-hickson").action(ArgAction::SetTrue).help("Escludi catalogo Hickson"))
        .arg(Arg::new("no-abell").long("no-abell").action(ArgAction::SetTrue).help("Escludi catalogo Abell"))
        .arg(Arg::new("no-doublestar").long("no-doublestar").action(ArgAction::SetTrue).help("Escludi catalogo Double Star"))
        .arg(Arg::new("merge-radius").long("merge-radius").value_name("ARCSEC").default_value("30.0").value_parser(clap::value_parser!(f64)).help("Raggio di merge per cross-correlazione in arcosecondi"))
}

// ===========================================================================
// catalog validate — validazione contro SIMBAD
// ===========================================================================

async fn cmd_catalog_validate(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let path = args.get_one::<String>("input").expect("required");
    let max = *args.get_one::<usize>("max").unwrap_or(&50);
    let tolerance = *args.get_one::<f64>("tolerance").unwrap_or(&60.0);
    let report_path = args.get_one::<String>("report").map(|s| s.as_str());

    let content = std::fs::read_to_string(path)?;
    let mut objects: Vec<import_catalogs::models::ImportCatalogEntry> = serde_json::from_str(&content)?;
    let total = objects.len();
    let n = if max == 0 { total } else { max.min(total) };
    println!("Validazione SIMBAD: {} oggetti su {} (tolleranza {} arcsec)...", n, total, tolerance);

    // Seleziona n oggetti casuali
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut rng_state = seed;
    for i in (1..total).rev() {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (rng_state >> 33) as usize % (i + 1);
        if j != i { objects.swap(i, j); }
    }
    let sample: Vec<_> = objects.into_iter().take(n).collect();

    let results = import_catalogs::validate::validate_against_simbad(&sample, n, tolerance).await;

    let ok = results.iter().filter(|r| r.status == "OK").count();
    let warn = results.iter().filter(|r| r.status == "WARN").count();
    let err = results.iter().filter(|r| r.status == "ERROR").count();
    println!("Risultati: {} OK, {} WARN, {} ERROR", ok, warn, err);

    for r in &results {
        if !r.errors.is_empty() {
            println!("  [ERR] {}: {}", r.name, r.errors.join("; "));
        } else if !r.warnings.is_empty() {
            println!("  [WARN] {}: {}", r.name, r.warnings.join("; "));
        }
    }

    if let Some(p) = report_path {
        std::fs::write(p, serde_json::to_string_pretty(&results)?)?;
        println!("Report scritto in {}", p);
    }
    Ok(())
}

fn catalog_validate_cmd() -> Command {
    Command::new("validate")
        .about("Valida il catalogo JSON interrogando SIMBAD (verifica coordinate e cross-referenze)")
        .arg(Arg::new("input").short('i').long("input").value_name("FILE").required(true).help("File JSON del catalogo"))
        .arg(Arg::new("max").long("max").value_name("N").default_value("50").value_parser(clap::value_parser!(usize)).help("Numero di oggetti da validare (0 = tutti)"))
        .arg(Arg::new("tolerance").long("tolerance").value_name("ARCSEC").default_value("60.0").value_parser(clap::value_parser!(f64)).help("Tolleranza coordinate in arcsecondi"))
        .arg(Arg::new("report").long("report").value_name("FILE").help("File JSON di report"))
}

// ===========================================================================
// catalog import — importa in MongoDB
// ===========================================================================

async fn cmd_catalog_import(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let path = args.get_one::<String>("input").expect("required");
    let match_path = args.get_one::<String>("match-file").map(|s| s.as_str()).unwrap_or("oggetti_import_match.json");
    let purge = !args.get_flag("no-purge");

    let cfg = MongoDBConfig::load().map_err(conf_error)?;
    let mongo = Arc::new(MongoDBService::new(&cfg).await?);
    OggettiAstronomiciModule::register(&mongo).await?;

    let content = std::fs::read_to_string(path)?;
    let objects: Vec<Value> = serde_json::from_str(&content)?;
    println!("Importazione di {} oggetti da {}", objects.len(), path);

    if purge {
        let col = mongo.db().collection::<MongoOggettoAstronomicoModel>(
            MongoOggettoAstronomicoModel::collection_name(),
        );

        // Trova oggetti imported con immagini da pulire
        let with_images: Vec<MongoOggettoAstronomicoModel> = col
            .find(bdoc! { "imported": true, "image_filename": bdoc! { "$exists": true } })
            .await?
            .try_collect()
            .await
            .map_err(|e| format!("query imported images: {e}"))?;

        let fm_conf = FileManagerConf::load().map_err(conf_error)?;
        let base_conf = cornetti::core::confs::BaseConf::load().map_err(conf_error)?;
        let ns = base_conf.shared_resources_id;
        let tenant = base_conf.tenant_id;

        for obj in &with_images {
            if let Some(ref img) = obj.image_filename {
                delete_oggetto_image(&mongo, img, &tenant, &ns, &fm_conf).await?;
            }
        }
        if !with_images.is_empty() {
            println!("Eliminate {} immagini associate a oggetti importati.", with_images.len());
        }

        let n = col.delete_many(bdoc! { "imported": true }).await?.deleted_count;
        println!("Rimossi {} oggetti precedentemente importati (imported=true)", n);
    }

    let collection = mongo.db().collection::<MongoOggettoAstronomicoModel>(
        MongoOggettoAstronomicoModel::collection_name(),
    );

    let existing: Vec<MongoOggettoAstronomicoModel> = collection
        .find(bdoc! { "imported": false }).await?
        .try_collect().await.map_err(|e| format!("Query error: {e}"))?;

    let mut conflicts: Vec<(String, Vec<String>)> = Vec::new();
    for ex in &existing {
        for ec in &ex.cataloghi {
            let ek = (ec.catalog_id.clone(), ec.catalog_nr.clone());
            for obj in &objects {
                if let Some(cats) = obj["cataloghi"].as_array() {
                    for nc in cats {
                        let nk = (nc["catalog_id"].as_str().unwrap_or("").to_uppercase(), nc["catalog_nr"].as_str().unwrap_or("").to_uppercase());
                        if ek == nk {
                            let s = format!("{}{}", ek.0, ek.1);
                            if let Some(e) = conflicts.iter_mut().find(|c| c.0 == s) {
                                if !e.1.contains(&s) { e.1.push(s.clone()); }
                            } else {
                                conflicts.push((ex._id.as_ref().map(|id| id.to_string()).unwrap_or_default(), vec![s]));
                            }
                        }
                    }
                }
            }
        }
    }
    if !conflicts.is_empty() {
        println!("ATTENZIONE: {} oggetti non importati (imported=false) hanno cataloghi in conflitto:", conflicts.len());
        for (id, cats) in &conflicts { println!("  {} -> {:?}", id, cats); }
    } else {
        println!("Nessun conflitto con oggetti non importati.");
    }

    let mut imported = 0u64;
    let mut skipped = 0u64;
    let mut match_entries: Vec<ImportMatchEntry> = Vec::new();

    for obj in &objects {
        let mut model = MongoOggettoAstronomicoModel::new();
        model.tipo = serde_json::from_value(obj["tipo"].clone()).unwrap_or_default();
        model.nome_comune = obj["nome_comune"].as_str().unwrap_or("").to_string();
        model.abbr_costellazione = serde_json::from_value(obj["abbr_costellazione"].clone()).unwrap_or_default();
        model.coord_ar = obj["coord_ar"].as_str().unwrap_or("").to_string();
        model.coord_dec = obj["coord_dec"].as_str().unwrap_or("").to_string();

        // Salta oggetti senza coordinate
        if model.coord_ar.is_empty() || model.coord_dec.is_empty() {
            skipped += 1;
            continue;
        }
        model.mag_apparente = obj["mag_apparente"].as_f64();
        model.note = obj["note"].as_str().unwrap_or("").to_string();
        model.multi = obj["multi"].as_bool().unwrap_or(false);
        model.imported = true;

        if let Some(dim) = obj.get("dim_apparenti") {
            model.dim_apparenti = Some(app_modules::astronomia::oggetti_astronomici::repos::MongoDimensioniApparentiModel {
                secs_a: dim["secs_a"].as_i64().map(|v| v as i32),
                secs_b: dim["secs_b"].as_i64().map(|v| v as i32),
                dms_a: dim["secs_a"].as_i64().map(|v| app_modules::astronomia::common::helpers::secs_to_dms_string(v as i32)),
                dms_b: dim["secs_b"].as_i64().map(|v| app_modules::astronomia::common::helpers::secs_to_dms_string(v as i32)),
                secs_rapp: match (dim["secs_a"].as_i64(), dim["secs_b"].as_i64()) { (Some(a), Some(b)) if a > 0 && b > 0 => Some(a * b), _ => None },
            });
        }

        let mut catalog_keys: Vec<CatalogMatchKey> = Vec::new();
        if let Some(cataloghi) = obj["cataloghi"].as_array() {
            for cat in cataloghi {
                let cid = cat["catalog_id"].as_str().unwrap_or("").to_uppercase().trim().to_string();
                let cnr = cat["catalog_nr"].as_str().unwrap_or("").to_uppercase().trim().to_string();
                model.cataloghi.push(app_modules::astronomia::oggetti_astronomici::repos::MongoCatalogoModel {
                    catalog_id: cid.clone(), catalog_nr: cnr.clone(),
                    extended: Some(format!("{} {}", cid, cnr).trim().to_string()),
                });
                catalog_keys.push(CatalogMatchKey { catalog_id: cid, catalog_nr: cnr });
            }
        }

        match collection.insert_one(&model).await {
            Ok(r) => {
                if let Some(oid) = r.inserted_id.as_object_id() {
                    match_entries.push(ImportMatchEntry { mongo_id: oid.to_hex(), cataloghi: catalog_keys });
                }
                imported += 1;
            }
            Err(e) => { log::warn!("Salto oggetto: {}", e); skipped += 1; }
        }
    }

    std::fs::write(match_path, serde_json::to_string_pretty(&match_entries)?)?;
    println!("Importazione completata: {} importati, {} saltati, {} conflitti", imported, skipped, conflicts.len());
    println!("File di match: {}", match_path);
    Ok(())
}

fn catalog_import_cmd() -> Command {
    Command::new("import")
        .about("Importa il catalogo JSON in MongoDB. Imposta imported=true. Elimina gli imported precedenti.")
        .arg(Arg::new("input").short('i').long("input").value_name("FILE").required(true).help("File JSON del catalogo"))
        .arg(Arg::new("match-file").long("match-file").value_name("FILE").default_value("oggetti_import_match.json").help("File di mapping catalogo -> mongo _id"))
        .arg(Arg::new("no-purge").long("no-purge").action(ArgAction::SetTrue).help("Non eliminare gli oggetti imported=true preesistenti"))
}

// ===========================================================================
// images generate — genera JSON immagini DSS
// ===========================================================================

async fn cmd_images_generate(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input = args.get_one::<String>("input").expect("required");
    let output = args.get_one::<String>("output").map(|s| s.as_str()).unwrap_or("oggetti_astronomici_images.json");
    let survey = args.get_one::<String>("survey").map(|s| s.as_str()).unwrap_or("poss2ukstu_red");
    let dldir = args.get_one::<String>("download-dir").map(|s| s.as_str());

    let content = std::fs::read_to_string(input)?;
    let objects: Vec<import_catalogs::models::ImportCatalogEntry> = serde_json::from_str(&content)?;

    let mut entries = import_images::dss::generate_dss_entries(&objects, survey, 15.0);
    println!("Generati {} URL immagini DSS", entries.len());

    if let Some(dir) = dldir {
        let dir = std::path::Path::new(dir);
        std::fs::create_dir_all(dir)?;
        for e in &mut entries {
            let p = dir.join(format!("dss_{:05}.gif", e.index));
            if p.exists() { e.local_path = Some(p.to_string_lossy().to_string()); continue; }
            match import_images::dss::download_dss_image(&e.url, &p).await {
                Ok(()) => e.local_path = Some(p.to_string_lossy().to_string()),
                Err(err) => log::warn!("Download fallito [{}]: {}", e.index, err),
            }
        }
        let ok = entries.iter().filter(|e| e.local_path.is_some()).count();
        println!("Scaricati {}/{} immagini", ok, entries.len());
    }

    std::fs::write(output, serde_json::to_string_pretty(&entries)?)?;
    println!("Immagini JSON scritto in {}", output);
    Ok(())
}

fn images_generate_cmd() -> Command {
    Command::new("generate")
        .about("Genera il file JSON con URL e metadati delle immagini DSS (Digitized Sky Survey) per ogni oggetto. Il FOV e' calcolato automaticamente dalle dimensioni apparenti.")
        .arg(Arg::new("input").short('i').long("input").value_name("FILE").required(true).help("File JSON del catalogo"))
        .arg(Arg::new("output").short('o').long("output").value_name("FILE").default_value("oggetti_astronomici_images.json").help("File JSON in output"))
        .arg(Arg::new("survey").long("survey").value_name("SURVEY").default_value("poss2ukstu_red").help("DSS survey (poss2ukstu_red, poss2ukstu_blue, poss1_red, ...)"))
        .arg(Arg::new("download-dir").long("download-dir").value_name("DIR").help("Scarica le immagini in questa directory"))
}

// ===========================================================================
// images import — importa immagini DSS in MongoDB
// ===========================================================================

async fn cmd_images_import(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let download_dir = args.get_one::<String>("download-dir").map(|s| s.as_str());
    let survey = args.get_one::<String>("survey").map(|s| s.as_str()).unwrap_or("poss2ukstu_red");
    let only_missing = args.get_flag("only-missing");

    let cfg = MongoDBConfig::load().map_err(conf_error)?;
    let mongo = Arc::new(MongoDBService::new(&cfg).await?);
    OggettiAstronomiciModule::register(&mongo).await?;
    FileManagerModule::register(&mongo).await?;

    // Carica tutti gli oggetti imported=true con coordinate
    let coll = mongo.db().collection::<bson::Document>(
        MongoOggettoAstronomicoModel::collection_name(),
    );

    let mut filter = bdoc! {
        "imported": true,
        "coord_ar": bdoc! { "$ne": "" },
        "coord_dec": bdoc! { "$ne": "" },
    };
    if only_missing {
        filter.insert("image_filename", bdoc! { "$exists": false });
    }

    let objects: Vec<bson::Document> = coll
        .find(filter)
        .await?
        .try_collect()
        .await
        .map_err(|e| format!("query oggetti: {e}"))?;

    println!("Oggetti da processare: {}", objects.len());

    let fm_conf = FileManagerConf::load().map_err(conf_error)?;
    let base_conf = cornetti::core::confs::BaseConf::load().map_err(conf_error)?;
    let ns = base_conf.shared_resources_id;
    let tenant = base_conf.tenant_id;
    let identity_email = std::env::var("APP_IMPORT_IDENTITY").unwrap_or_else(|_| "import@system".into());

    // Risolvi identity_id (ObjectId) cercando l'utente per email
    let users_coll = mongo.db().collection::<bson::Document>("users");
    let user_doc = users_coll
        .find_one(bdoc! { "email": &identity_email })
        .await
        .map_err(|e| format!("query utente {}: {}", identity_email, e))?
        .ok_or_else(|| format!("Utente '{}' non trovato. Crea l'utente prima di importare immagini.", identity_email))?;
    let identity_id = user_doc
        .get_object_id("_id")
        .map_err(|_| "Formato _id utente non valido".to_string())?
        .to_hex();

    let fm_repo = app_modules::base::filemanager::repos::FileManagerRepository::new(mongo.clone());
    let oggetti_repo = app_modules::astronomia::oggetti_astronomici::repos::OggettiAstronomiciRepository::new(mongo.clone());

    let mut uploaded = 0u64;
    let mut skipped = 0u64;
    let mut failed = 0u64;
    let total = objects.len();
    let start = std::time::Instant::now();

    for (i, obj) in objects.iter().enumerate() {
        use std::io::Write;
        let pct = (i + 1) * 100 / total;
        let elapsed = start.elapsed().as_secs();
        print!("\r  [{}/{}] {}% | upload={} fail={} skip={} | {}s", i + 1, total, pct, uploaded, failed, skipped, elapsed);
        let _ = std::io::stdout().flush();

        let mongo_id = obj.get_object_id("_id")
            .map_err(|_| "_id non trovato".to_string())?
            .to_hex();

        let coord_ar = obj.get_str("coord_ar").unwrap_or("");
        let coord_dec = obj.get_str("coord_dec").unwrap_or("");
        if coord_ar.is_empty() || coord_dec.is_empty() { skipped += 1; continue; }

        // Leggi dimensioni apparenti per FOV dinamico
        let secs_a = obj.get("dim_apparenti")
            .and_then(|d| d.as_document())
            .and_then(|d| d.get_i32("secs_a").ok());
        let secs_b = obj.get("dim_apparenti")
            .and_then(|d| d.as_document())
            .and_then(|d| d.get_i32("secs_b").ok());
        let fov = import_images::dss::compute_fov(secs_a, secs_b);

        let url = import_images::dss::build_dss_url(coord_ar, coord_dec, survey, fov);

        // Se l'oggetto ha gia' un'immagine, elimina la precedente
        if let Some(old_img) = obj.get_str("image_filename").ok() {
            if !old_img.is_empty() {
                delete_oggetto_image(&mongo, old_img, &tenant, &ns, &fm_conf).await?;
                log::info!("  Immagine precedente eliminata: {}", old_img);
            }
        }

        // Scarica immagine
        let dl_dir = download_dir.unwrap_or("/tmp/dss_downloads");
        let dest = std::path::PathBuf::from(dl_dir).join(format!("dss_{}.gif", &mongo_id[..8]));

        match import_images::dss::download_dss_image(&url, &dest).await {
            Ok(()) => {}
            Err(e) => { log::warn!("Download fallito [{}]: {}", &mongo_id[..8], e); failed += 1; continue; }
        }

        // Upload via filemanager
        let meta = std::fs::metadata(&dest)?;
        let fc = match upload_file_from_path(
            &dest, "dss_image.gif", meta.len() as usize,
            &fm_conf.allowed_file_types, &fm_conf.upload_directory,
            &tenant, &ns, &identity_email, &identity_id,
            Some(app_modules::astronomia::common::TYPE_ASTRO_OBJECT_IMAGE), None,
        ) {
            Ok(f) => f,
            Err(e) => { log::warn!("upload fallito [{}]: {}", &mongo_id[..8], e.detail); failed += 1; continue; }
        };
        let fnm = fc.filename.clone();
        let _ = fm_repo.create(&tenant, fc).await.map_err(|e| format!("fm create: {}", e.detail));

        // Linka l'immagine all'oggetto
        let caption = format!("DSS {} (FOV {}')", survey, fov);
        oggetti_repo.set_image_fields(&mongo_id, &fnm, Some(&caption), Some(fov)).await
            .map_err(|e| format!("set_image_fields [{}]: {}", &mongo_id[..8], e.detail))?;
        uploaded += 1;
    }
    println!();
    let elapsed = start.elapsed().as_secs();
    println!("Import immagini: {} caricate, {} saltate, {} fallite ({}s)", uploaded, skipped, failed, elapsed);
    Ok(())
}

fn images_import_cmd() -> Command {
    Command::new("import")
        .about("Scarica le immagini DSS per tutti gli oggetti imported=true e le associa via filemanager. Il FOV e' calcolato automaticamente dalle dimensioni apparenti.")
        .arg(Arg::new("download-dir").long("download-dir").value_name("DIR").default_value("/tmp/dss_downloads").help("Directory per download temporanei"))
        .arg(Arg::new("survey").long("survey").value_name("SURVEY").default_value("poss2ukstu_red").help("DSS survey (poss2ukstu_red, poss2ukstu_blue, ...)"))
        .arg(Arg::new("only-missing").long("only-missing").action(ArgAction::SetTrue).help("Scarica solo oggetti senza immagine (image_filename non presente)"))
}

// ===========================================================================
// admin create — crea gruppo admin + utente amministratore
// ===========================================================================

async fn cmd_admin_create() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = MongoDBConfig::load().map_err(conf_error)?;
    let mongo = Arc::new(MongoDBService::new(&cfg).await?);
    register_all_modules(&mongo).await?;

    use app_modules::base::users::repos::UsersRepository;
    use app_modules::base::users::models::{UserCreate, UserUpdate, SetPasswordBody};

    let users_repo = UsersRepository::new(mongo.clone());

    // --- Verifica/esiste gruppo "admins" via query diretta ---
    let groups_coll = mongo.db().collection::<bson::Document>("groups");
    let group_doc = groups_coll.find_one(bdoc! { "name": "admins" }).await?;

    let group_id = match group_doc {
        Some(ref g) => {
            println!("Gruppo 'admins' gia' esistente.");
            let id = g.get_object_id("_id").expect("no _id");
            id.to_hex()
        }
        None => {
            use bson::oid::ObjectId;

            let oid = ObjectId::new();
            let perm = bdoc! {
                "name": "all",
                "read": true,
                "create": true,
                "modify": true,
                "delete": true,
            };
            let doc = bdoc! {
                "_id": oid,
                "name": "admins",
                "description": "Amministratori di sistema – permessi completi su tutti i moduli",
                "default": false,
                "permissions": [perm],
                "created": bson::DateTime::now(),
                "modified": bson::DateTime::now(),
            };
            groups_coll.insert_one(doc).await?;
            println!("Gruppo 'admins' creato con id={}", oid.to_hex());
            oid.to_hex()
        }
    };

    // --- Prompt per email e password ---
    use console::Term;
    let term = Term::stdout();

    term.write_str("Email amministratore: ")?;
    let email = term.read_line()?.trim().to_string();
    if email.is_empty() { eprintln!("Email non valida"); return Ok(()); }

    term.write_str("Password (min 8 caratteri): ")?;
    let password = term.read_secure_line()?.trim().to_string();
    if password.len() < 8 { eprintln!("Errore: la password deve essere almeno di 8 caratteri"); return Ok(()); }

    term.write_str("Conferma password: ")?;
    let confirm = term.read_secure_line()?.trim().to_string();
    if password != confirm { eprintln!("Errore: le password non coincidono"); return Ok(()); }

    // --- Controlla se utente gia' esiste ---
    if let Some(existing) = users_repo.get_by_email(&email).await.map_err(|e| format!("get_by_email: {}", e.detail))? {
        let uid = existing.id.clone().unwrap_or_default();
        println!("Utente '{}' gia' esistente (id={}). Associazione al gruppo 'admins'.", email, uid);
        if let Some(_uid) = existing.id {
            let mut gids = existing.groups_ids.clone();
            if !gids.contains(&group_id) { gids.push(group_id); }
            let upd = UserUpdate {
                name: existing.name.unwrap_or_default(),
                surname: existing.surname.unwrap_or_default(),
                enabled: true,
                groups_ids: gids,
            };
            users_repo.update(&_uid, &upd).await.map_err(|e| format!("update user: {}", e.detail))?;
        }
        println!("Fatto.");
        return Ok(());
    }

    // --- Crea utente ---
    let nome = email.split('@').next().unwrap_or("admin").to_string();

    let uc = UserCreate {
        name: nome,
        surname: "Amministratore".into(),
        email: email.clone(),
        enabled: true,
        groups_ids: vec![group_id],
    };
    uc.validate()?;
    let user = users_repo.create(uc).await.map_err(|e| format!("create user: {}", e.detail))?;
    let uid = user.id.unwrap();

    let sp = SetPasswordBody { password: password.clone(), confirm_password: password.clone() };
    sp.validate()?;
    users_repo.set_password(&uid, &password).await.map_err(|e| format!("set_password: {}", e.detail))?;

    println!("Utente amministratore creato: {} (id={})", email, uid);
    println!("Il gruppo 'admins' ha permessi completi su tutti i moduli.");
    Ok(())
}

fn admin_create_cmd() -> Command {
    Command::new("create")
        .about("Crea il gruppo 'admins' con permessi totali e un utente amministratore. Chiede email e password interattivamente.")
}

fn build_cli() -> Command {
    Command::new("app_managment")
        .about("Gestione dell'applicazione: registrazione moduli, import cataloghi astronomici, validazione SIMBAD, immagini DSS, utenti.")

        // Top-level flags
        .arg(Arg::new("register-modules")
            .short('r').long("register-modules")
            .help("Registra tutti i moduli in MongoDB (crea collezioni e indici)")
            .action(ArgAction::SetTrue))

        // Subcommands
        .subcommand(Command::new("catalog")
            .about("Operazioni sui cataloghi astronomici")
            .subcommand_required(true)
            .subcommand(catalog_generate_cmd())
            .subcommand(catalog_validate_cmd())
            .subcommand(catalog_import_cmd()))

        .subcommand(Command::new("images")
            .about("Operazioni sulle immagini DSS")
            .subcommand_required(true)
            .subcommand(images_generate_cmd())
            .subcommand(images_import_cmd()))

        .subcommand(Command::new("admin")
            .about("Gestione amministratori")
            .subcommand_required(true)
            .subcommand(admin_create_cmd()))

        .subcommand(tests::tests_cmd())
        .subcommand(redis_cache::redis_cmd())
}

// ===========================================================================
// Dispatch
// ===========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing_log::LogTracer::init().ok();

    let mut cli = build_cli();
    let matches = cli.clone().get_matches();

    if matches.get_flag("register-modules") {
        let cfg = MongoDBConfig::load().map_err(conf_error)?;
        let mongo = MongoDBService::new(&cfg).await?;
        return register_all_modules(&mongo).await;
    }

    // Subcommands
    match matches.subcommand() {
        Some(("catalog", sub)) => match sub.subcommand() {
            Some(("generate", a)) => cmd_catalog_generate(a).await,
            Some(("validate", a)) => cmd_catalog_validate(a).await,
            Some(("import", a)) => cmd_catalog_import(a).await,
            _ => { let _ = cli.print_help(); Ok(()) }
        },
        Some(("images", sub)) => match sub.subcommand() {
            Some(("generate", a)) => cmd_images_generate(a).await,
            Some(("import", a)) => cmd_images_import(a).await,
            _ => { let _ = cli.print_help(); Ok(()) }
        },
        Some(("admin", sub)) => match sub.subcommand() {
            Some(("create", _)) => cmd_admin_create().await,
            _ => { let _ = cli.print_help(); Ok(()) }
        },
        Some(("test", sub)) => tests::dispatch(sub).await,
        Some(("redis", sub)) => redis_cache::dispatch(sub).await,
        _ => { let _ = cli.print_help(); Ok(()) }
    }
}
