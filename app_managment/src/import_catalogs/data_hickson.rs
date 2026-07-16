use crate::import_catalogs::coords::*;
use crate::import_catalogs::models::*;
use std::collections::HashMap;

pub fn load_hickson() -> (Vec<ImportCatalogEntry>, Vec<ImportCatalogEntry>) {
    let groups: HashMap<i32, (Vec<(i32, &str)>, &str, i32, i32, i64, char, i32, i32, i32)> = HashMap::from([
(1, (vec![(7803,"a"),(7806,"c")], "Psc", 0,1,0, '+', 6,8,0)),
(2, (vec![(3630,"a"),(3632,"b"),(3633,"c")], "Leo", 11,15,30, '+', 3,14,0)),
(3, (vec![(3974,"a"),(3976,"b")], "Crt", 11,56,0, '-', 17,0,0)),
(4, (vec![(5836,"a"),(5837,"b"),(5838,"c")], "Lib", 15,1,30, '-', 14,40,0)),
(7, (vec![(192,"a"),(196,"b"),(197,"c"),(201,"d")], "Cet", 0,39,36, '+', 0,50,0)),
(10, (vec![(536,"a"),(542,"c"),(543,"d")], "And", 1,26,0, '+', 34,42,0)),
(16, (vec![(833,"b"),(838,"c"),(839,"d")], "Cet", 2,10,0, '-', 10,10,0)),
(21, (vec![(1099,"a"),(1100,"b"),(1101,"c"),(1102,"d"),(1103,"e"),(1104,"f"),(1105,"g")], "Eri", 2,48,0, '-', 17,30,0)),
(22, (vec![(1190,"a"),(1191,"b"),(1192,"c"),(1199,"d")], "Eri", 3,5,0, '-', 15,30,0)),
(23, (vec![(1214,"a"),(1215,"b"),(1216,"c")], "Eri", 3,7,0, '-', 9,0,0)),
(27, (vec![(1353,"a")], "Eri", 3,32,0, '-', 21,0,0)),
(29, (vec![(1241,"b"),(1242,"c"),(1247,"d")], "Eri", 3,12,0, '-', 6,40,0)),
(42, (vec![(3091,"a"),(3096,"b")], "Sex", 10,1,0, '+', 19,10,0)),
(44, (vec![(3185,"a"),(3187,"b"),(3190,"c"),(3193,"d")], "Leo", 10,18,0, '+', 22,0,0)),
(56, (vec![(3717,"a"),(3720,"b"),(3745,"c"),(3746,"d"),(3748,"e"),(3750,"f"),(3753,"g")], "Leo", 11,30,0, '+', 22,0,0)),
(57, (vec![(3754,"a"),(3755,"b")], "Leo", 11,35,0, '+', 22,0,0)),
(58, (vec![(3822,"a"),(3825,"b"),(3826,"c"),(3832,"d")], "Vir", 11,42,0, '+', 10,10,0)),
(61, (vec![(4169,"a"),(4173,"b"),(4174,"c"),(4175,"d")], "Com", 12,12,15, '+', 29,12,0)),
(62, (vec![(4776,"a"),(4778,"b"),(4779,"c"),(4780,"d"),(4782,"e"),(4784,"f")], "Vir", 12,52,0, '-', 9,15,0)),
(67, (vec![(5298,"a"),(5299,"b"),(5300,"c"),(5302,"d"),(5303,"e"),(5306,"f")], "Vir", 13,47,0, '-', 29,30,0)),
(68, (vec![(5350,"a"),(5353,"b"),(5354,"c"),(5355,"d"),(5358,"e")], "CVn", 13,53,0, '+', 40,20,0)),
(71, (vec![(5004,"a")], "CVn", 13,10,0, '+', 39,0,0)),
(73, (vec![(5829,"a")], "Boo", 15,2,30, '+', 23,0,0)),
(74, (vec![(5910,"a")], "Ser", 15,17,0, '+', 17,30,0)),
(79, (vec![(6027,"a"),(6028,"b"),(6029,"c")], "Ser", 15,59,0, '+', 20,45,0)),
(88, (vec![(6975,"a"),(6976,"b"),(6977,"c"),(6978,"d")], "Aqr", 20,52,0, '-', 5,50,0)),
(90, (vec![(7172,"a"),(7173,"b"),(7174,"c"),(7175,"f"),(7176,"e")], "PsA", 22,2,0, '-', 32,0,0)),
(91, (vec![(7214,"a")], "PsA", 22,4,0, '-', 27,50,0)),
(92, (vec![(7317,"a"),(7318,"b"),(7319,"c"),(7320,"d")], "Peg", 22,36,0, '+', 34,0,0)),
(93, (vec![(7550,"a"),(7552,"b"),(7554,"c")], "Peg", 23,16,0, '+', 9,24,0)),
(94, (vec![(7578,"a")], "Peg", 23,19,0, '+', 18,40,0)),
(95, (vec![(7609,"a")], "Psc", 23,20,0, '+', 8,4,0)),
(96, (vec![(7674,"a")], "Peg", 23,28,0, '+', 8,48,0)),
(100,(vec![(7803,"a"),(7806,"b")], "Psc", 0,1,30, '+', 13,10,0)),
    ]);

    let mut individuals = Vec::new();
    let mut group_entries = Vec::new();

    for (&group_nr, (members, con, ra_h, ra_m, ra_s, dec_sign, dec_d, dec_m, dec_s)) in &groups {
        let ra = ra_hms_to_decimal(*ra_h as f64, *ra_m as f64, *ra_s as f64);
        let dec = dec_dms_to_decimal(*dec_sign, *dec_d as f64, *dec_m as f64, *dec_s as f64);

        let n_membri = members.len();

        for &(ngc, letter) in members {
            let catalog_nr = format!("{group_nr}-{letter}");
            individuals.push(ImportCatalogEntry {
                cataloghi: vec![
                    CatalogEntry { catalog_id: "HCG".into(), catalog_nr: catalog_nr },
                    CatalogEntry { catalog_id: "NGC".into(), catalog_nr: ngc.to_string() },
                ],
                tipo: "GAL".to_string(),
                nome_comune: String::new(),
                abbr_costellazione: Costellazione::parse(con),
                coord_ar: decimal_ra_to_coord(ra),
                coord_dec: decimal_dec_to_coord(dec),
                mag_apparente: None,
                dim_apparenti: None,
                note: format!("HCG {group_nr}{letter}"),
                multi: false,
                ra_decimal: Some(ra),
                dec_decimal: Some(dec),
            });
        }

        group_entries.push(ImportCatalogEntry {
            cataloghi: vec![
                CatalogEntry { catalog_id: "HCG".into(), catalog_nr: group_nr.to_string() },
            ],
            tipo: "HCG".to_string(),
            nome_comune: String::new(),
            abbr_costellazione: Costellazione::parse(con),
            coord_ar: decimal_ra_to_coord(ra),
            coord_dec: decimal_dec_to_coord(dec),
            mag_apparente: None,
            dim_apparenti: None,
            note: format!("HCG {group_nr}: {n_membri} membri"),
            multi: true,
            ra_decimal: Some(ra),
            dec_decimal: Some(dec),
        });
    }

    (individuals, group_entries)
}
