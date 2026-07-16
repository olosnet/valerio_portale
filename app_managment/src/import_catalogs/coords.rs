/// Coordinate conversion utilities for astronomical catalogs.
/// Mirrors the Python coordinate helpers used in the import scripts.

pub fn ra_hms_to_decimal(hours: f64, minutes: f64, seconds: f64) -> f64 {
    15.0 * (hours + minutes / 60.0 + seconds / 3600.0)
}

pub fn dec_dms_to_decimal(sign: char, degrees: f64, minutes: f64, seconds: f64) -> f64 {
    let val = degrees.abs() + minutes / 60.0 + seconds / 3600.0;
    if sign == '-' {
        -val
    } else {
        val
    }
}

pub fn decimal_ra_to_coord(ra_decimal: f64) -> String {
    let ra = ra_decimal.rem_euclid(360.0);
    let hours_total = ra / 15.0;
    let h = hours_total as i32;
    let m = ((hours_total - h as f64) * 60.0) as i32;
    let s = ((hours_total - h as f64) * 60.0 - m as f64) * 60.0;
    let secs_int = s as i32;
    let secs_frac = ((s - secs_int as f64) * 10.0).round() as i32;
    let (h, m, secs_int, secs_frac) = normalize_ra_parts(h, m, secs_int, secs_frac);
    if secs_frac > 0 {
        format!("{h:02}{m:02}{secs_int:02}.{secs_frac}")
    } else {
        format!("{h:02}{m:02}{secs_int:02}")
    }
}

fn normalize_ra_parts(mut h: i32, mut m: i32, mut s: i32, mut f: i32) -> (i32, i32, i32, i32) {
    if f == 10 { s += 1; f = 0; }
    if s == 60 { s = 0; m += 1; }
    if m == 60 { m = 0; h += 1; }
    (h, m, s, f)
}

pub fn decimal_dec_to_coord(dec_decimal: f64) -> String {
    let sign = if dec_decimal >= 0.0 { '+' } else { '-' };
    let dec = dec_decimal.abs();
    let d = dec as i32;
    let m = ((dec - d as f64) * 60.0) as i32;
    let s = (((dec - d as f64) * 60.0 - m as f64) * 60.0) as i32;
    format!("{sign}{d:02}{m:02}{s:02}")
}

pub fn angular_separation_arcsec(ra1: f64, dec1: f64, ra2: f64, dec2: f64) -> f64 {
    let dra_deg = (ra1 - ra2) * ((dec1 + dec2) / 2.0).to_radians().cos();
    let ddec_deg = dec1 - dec2;
    3600.0 * (dra_deg * dra_deg + ddec_deg * ddec_deg).sqrt()
}
