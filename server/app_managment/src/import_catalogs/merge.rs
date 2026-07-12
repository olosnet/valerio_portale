use std::collections::HashSet;

use crate::import_catalogs::coords::angular_separation_arcsec;
use crate::import_catalogs::models::*;

fn _dedup_catalogs(cats: Vec<CatalogEntry>) -> Vec<CatalogEntry> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    cats.into_iter()
        .filter(|c| seen.insert((c.catalog_id.clone(), c.catalog_nr.clone())))
        .collect()
}

fn _merge_into(target: &mut ImportCatalogEntry, source: &ImportCatalogEntry) {
    target.cataloghi.extend(source.cataloghi.clone());
    target.cataloghi = _dedup_catalogs(std::mem::take(&mut target.cataloghi));

    if target.mag_apparente.is_none() {
        target.mag_apparente = source.mag_apparente;
    }
    if target.nome_comune.is_empty() && !source.nome_comune.is_empty() {
        target.nome_comune = source.nome_comune.clone();
    }
    if target.abbr_costellazione.is_empty() && !source.abbr_costellazione.is_empty() {
        target.abbr_costellazione = source.abbr_costellazione.clone();
    }
    if target.tipo == "GAL" && source.tipo != "GAL" {
        target.tipo = source.tipo.clone();
    }
    if source.multi {
        target.multi = true;
    }
    if target.ra_decimal.is_none() {
        target.ra_decimal = source.ra_decimal;
    }
    if target.dec_decimal.is_none() {
        target.dec_decimal = source.dec_decimal;
    }
}

fn _has_same_ngc_ic(a: &ImportCatalogEntry, b: &ImportCatalogEntry) -> bool {
    for ca in &a.cataloghi {
        if ca.catalog_id != "NGC" && ca.catalog_id != "IC" {
            continue;
        }
        for cb in &b.cataloghi {
            if cb.catalog_id != "NGC" && cb.catalog_id != "IC" {
                continue;
            }
            if ca.catalog_id == cb.catalog_id && ca.catalog_nr == cb.catalog_nr {
                return true;
            }
        }
    }
    false
}

fn _has_same_catalog_key(a: &ImportCatalogEntry, b: &ImportCatalogEntry) -> bool {
    for ca in &a.cataloghi {
        for cb in &b.cataloghi {
            if ca.catalog_id == cb.catalog_id && ca.catalog_nr == cb.catalog_nr {
                return true;
            }
        }
    }
    false
}

pub fn merge_objects(
    objects: Vec<ImportCatalogEntry>,
    match_radius_arcsec: f64,
) -> Vec<ImportCatalogEntry> {
    let mut objects = objects;

    // Fase 1: merge per numero NGC/IC
    let mut i = 0;
    while i < objects.len() {
        let mut j = i + 1;
        while j < objects.len() {
            if !objects[i].multi && !objects[j].multi && _has_same_ngc_ic(&objects[i], &objects[j]) {
                let source = objects.remove(j);
                _merge_into(&mut objects[i], &source);
            } else {
                j += 1;
            }
        }
        i += 1;
    }

    // Fase 2: merge per prossimità coordinate
    let mut i = 0;
    while i < objects.len() {
        if objects[i].multi {
            i += 1;
            continue;
        }
        let (ra_i, dec_i) = match (objects[i].ra_decimal, objects[i].dec_decimal) {
            (Some(r), Some(d)) => (r, d),
            _ => {
                i += 1;
                continue;
            }
        };
        let mut j = i + 1;
        while j < objects.len() {
            if objects[j].multi {
                j += 1;
                continue;
            }
            let (ra_j, dec_j) = match (objects[j].ra_decimal, objects[j].dec_decimal) {
                (Some(r), Some(d)) => (r, d),
                _ => {
                    j += 1;
                    continue;
                }
            };
            if angular_separation_arcsec(ra_i, dec_i, ra_j, dec_j) <= match_radius_arcsec {
                let source = objects.remove(j);
                _merge_into(&mut objects[i], &source);
            } else {
                j += 1;
            }
        }
        i += 1;
    }

    // Dedup oggetti multi per catalog key
    let mut i = 0;
    while i < objects.len() {
        if !objects[i].multi {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < objects.len() {
            if objects[j].multi && _has_same_catalog_key(&objects[i], &objects[j]) {
                objects.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }

    // Ordina: oggetti singoli prima, poi multi
    let mut singles = Vec::new();
    let mut multis = Vec::new();
    for o in objects {
        if o.multi {
            multis.push(o);
        } else {
            singles.push(o);
        }
    }
    singles.append(&mut multis);
    singles
}
