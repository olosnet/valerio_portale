use crate::import_catalogs::coords::*;
use crate::import_catalogs::models::*;

/// 50 well-known double/multiple stars from WDS/STF.
pub fn load_doublestar() -> Vec<ImportCatalogEntry> {
    let data: &[(&str, &str, &str, i32, i32, i64, char, i32, i32, i32, f64, &str, &str)] = &[
("DS1","3062","Cas",0,0,54,'+',63,10,0,8.2,"Almach / Gamma And","3"),
("DS2","2026","UMa",16,12,0,'+',54,32,0,5.3,"Alcor-Mizar / 80 UMa","2"),
("DS3","1523","Leo",11,5,24,'+',20,19,0,6.0,"Algieba / Gamma Leo","2"),
("DS4","1166","Cnc",7,58,0,'+',18,22,0,5.3,"Tegmine / Zeta Cnc","3"),
("DS5","2052","UMa",16,22,0,'+',29,56,0,6.1,"Xi UMa","2"),
("DS6","2272","Lyr",18,5,12,'+',30,32,0,6.7,"Epsilon Lyr (Dbl-Dbl)","4"),
("DS7","1744","Vir",13,19,48,'-',5,38,0,5.3,"Porrima / Gamma Vir","2"),
("DS8","1865","Boo",14,39,0,'+',27,4,0,4.5,"Izar / Epsilon Boo","2"),
("DS9","1909","Boo",14,50,0,'+',19,30,0,5.0,"Xi Boo","2"),
("DS10","60","Psc",0,46,0,'+',3,48,0,5.7,"Alrescha / Alpha Psc","2"),
("DS11","401","Ori",3,46,0,'+',23,54,0,5.3,"Meissa / Lambda Ori","2"),
("DS12","774","Ori",5,30,42,'-',5,53,0,5.1,"Rigel / Beta Ori","2"),
("DS13","738","Ori",5,29,48,'-',1,6,0,2.9,"Mintaka / Delta Ori","2"),
("DS14","752","Ori",5,32,18,'-',0,16,0,1.9,"Alnitak / Zeta Ori","2"),
("DS15","3121","Cyg",0,5,0,'+',32,0,0,8.0,"61 Cyg","2"),
("DS16","1196","Cnc",8,12,0,'+',17,38,0,4.2,"Zeta Cnc","3"),
("DS17","817","Mon",6,7,0,'-',6,18,0,4.7,"Beta Mon","3"),
("DS18","900","Pup",6,20,0,'-',43,10,0,5.1,"Kappa Pup","2"),
("DS19","1273","UMa",8,42,24,'+',48,30,0,3.8,"Alioth / Epsilon UMa","2"),
("DS20","1694","UMa",12,54,0,'+',56,0,0,4.3,"Nu UMa","2"),
("DS21","2289","Boo",18,0,0,'+',29,0,0,5.4,"Delta Boo","2"),
("DS22","1962","Hya",15,28,30,'+',6,48,0,6.0,"54 Hya","2"),
("DS23","2084","Lyr",17,18,36,'+',37,18,0,6.2,"17 Lyr","2"),
("DS24","2383","Lyr",18,44,24,'+',37,36,0,5.6,"Epsilon Lyr","2"),
("DS25","2486","Sco",19,5,36,'-',26,36,0,3.0,"Graffias / Beta Sco","2"),
("DS26","1932","Boo",14,44,0,'+',10,54,0,5.8,"Mu Boo","3"),
("DS27","2107","Boo",16,53,42,'+',38,6,0,6.2,"Xi Boo","2"),
("DS28","2605","Sgr",19,46,0,'-',19,12,0,5.3,"54 Sgr","2"),
("DS29","2727","Cap",20,36,0,'-',12,36,0,4.2,"Algedi / Alpha Cap","2"),
("DS30","2822","Cep",21,44,0,'+',58,48,0,4.5,"Delta Cep","2"),
("DS31","2863","Peg",22,12,0,'+',13,0,0,5.6,"37 Peg","2"),
("DS32","2909","Lac",22,20,0,'+',39,42,0,5.3,"8 Lac","4"),
("DS33","61","Psc",0,52,0,'+',6,30,0,5.3,"Zeta Psc","2"),
("DS34","280","Cet",2,39,6,'+',0,10,0,6.0,"37 Cet","2"),
("DS35","331","Per",2,56,42,'+',32,24,0,5.3,"Eta Per","2"),
("DS36","464","Tau",3,54,30,'+',10,0,0,5.0,"Theta2 Tau","2"),
("DS37","559","Aur",4,56,0,'+',33,10,0,5.0,"Theta Aur","2"),
("DS38","762","Gem",6,1,0,'+',23,36,0,5.1,"Nu Gem","2"),
("DS39","963","Gem",6,37,48,'+',20,18,0,3.5,"Propus / Eta Gem","2"),
("DS40","730","Ori",5,36,30,'-',1,58,0,2.0,"Trapezium / Theta1 Ori","4"),
("DS41","1555","Com",12,27,0,'+',26,42,0,5.3,"24 Com","2"),
("DS42","1768","CVn",13,42,30,'+',28,24,0,5.8,"25 CVn","2"),
("DS43","2140","CrB",17,12,0,'+',33,54,0,5.5,"Sigma CrB","2"),
("DS44","2375","Oph",18,0,0,'+',8,12,0,5.2,"70 Oph","2"),
("DS45","2525","Ser",19,38,0,'+',3,0,0,5.1,"Theta Ser","2"),
("DS46","2758","Aqr",20,59,0,'-',5,48,0,5.4,"94 Aqr","2"),
("DS47","3050","Cas",23,52,0,'+',56,0,0,5.0,"Sigma Cas","2"),
("DS48","3049","And",23,49,54,'+',46,36,0,5.3,"Pi And","2"),
("DS49","3105","And",0,30,0,'+',42,54,0,5.6,"52 And","2"),
("DS50","3116","Pup",7,6,0,'-',19,12,0,5.8,"2 Pup","2"),
    ];

    data.iter().map(|&(ds_id, stf_id, con, rh, rm, rs, dsign, dd, dm, ds, mag, name, n_components)| {
        let ra = ra_hms_to_decimal(rh as f64, rm as f64, rs as f64);
        let dec = dec_dms_to_decimal(dsign, dd as f64, dm as f64, ds as f64);
        let mut cataloghi = vec![
            CatalogEntry { catalog_id: "DS".into(), catalog_nr: ds_id.to_string() },
        ];
        if !stf_id.is_empty() {
            cataloghi.push(CatalogEntry { catalog_id: "STF".into(), catalog_nr: stf_id.to_string() });
        }
        ImportCatalogEntry {
            cataloghi,
            tipo: "2Star".to_string(),
            nome_comune: name.to_string(),
            abbr_costellazione: Costellazione::parse(con),
            coord_ar: decimal_ra_to_coord(ra),
            coord_dec: decimal_dec_to_coord(dec),
            mag_apparente: Some(mag),
            dim_apparenti: None,
            note: format!("{name} ({n_components} componenti)"),
            multi: true,
            ra_decimal: Some(ra),
            dec_decimal: Some(dec),
        }
    }).collect()
}
