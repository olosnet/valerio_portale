pub fn secs_to_dms(secs: i32) -> (i32, i32, i32) {
    let degrees = secs / 3600;
    let remainder = secs % 3600;
    let minutes = remainder / 60;
    let seconds = remainder % 60;

    (degrees, minutes, seconds)
}

pub fn secs_to_dms_string(value: i32) -> String {
    let (degrees, minutes, seconds) = secs_to_dms(value);
    format!("{:02}{:02}{:02}", degrees, minutes, seconds)
}

pub fn format_ar(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return s.to_string();
    }
    for hour_len in 1..=2 {
        for minute_len in 1..=2 {
            if s.len() <= hour_len + minute_len {
                continue;
            }
            let (hour_part, rem) = s.split_at(hour_len);
            let (minute_part, second_part) = rem.split_at(minute_len);
            if let (Ok(hours), Ok(minutes)) = (hour_part.parse::<u8>(), minute_part.parse::<u8>()) {
                if hours < 24 && minutes < 60 {
                    if let Ok(seconds) = second_part.parse::<f64>() {
                        if seconds < 60.0 {
                            let sec_str = if seconds.fract() == 0.0 {
                                format!("{:02}", seconds as u8)
                            } else {
                                format!("{:.1}", seconds)
                            };
                            return format!("{:02}h {:02}m {}s", hours, minutes, sec_str);
                        }
                    }
                }
            }
        }
    }
    s.to_string()
}

pub fn format_dec(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return s.to_string();
    }
    let (sign, rest) = match s.chars().next() {
        Some('+') => ("+", &s[1..]),
        Some('-') => ("-", &s[1..]),
        _ => return s.to_string(),
    };
    for deg_len in 1..=2 {
        for min_len in 1..=2 {
            if rest.len() <= deg_len + min_len {
                continue;
            }
            let (deg_part, rem) = rest.split_at(deg_len);
            let (min_part, sec_part) = rem.split_at(min_len);
            if let (Ok(degrees), Ok(minutes)) = (deg_part.parse::<u8>(), min_part.parse::<u8>()) {
                if degrees <= 90 && minutes < 60 {
                    if let Ok(seconds) = sec_part.parse::<f64>() {
                        if seconds < 60.0 {
                            let sec_str = if seconds.fract() == 0.0 {
                                format!("{:02}", seconds as u8)
                            } else {
                                format!("{:.1}", seconds)
                            };
                            return format!("{} {:02}° {:02}' {}''", sign, degrees, minutes, sec_str);
                        }
                    }
                }
            }
        }
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_ar_basic() {
        assert_eq!(format_ar("053516.5"), "05h 35m 16.5s");
    }

    #[test]
    fn format_ar_short_hour() {
        assert_eq!(format_ar("62244.3"), "06h 22m 44.3s");
    }

    #[test]
    fn format_ar_integer_seconds() {
        assert_eq!(format_ar("123011.5"), "01h 23m 11.5s");
    }

    #[test]
    fn format_ar_empty() {
        assert_eq!(format_ar(""), "");
    }

    #[test]
    fn format_ar_invalid() {
        assert_eq!(format_ar("invalid"), "invalid");
    }

    #[test]
    fn format_dec_basic() {
        assert_eq!(format_dec("-052322"), "- 05° 23' 22''");
    }

    #[test]
    fn format_dec_positive() {
        assert_eq!(format_dec("+123000"), "+ 01° 23' 00''");
    }

    #[test]
    fn format_dec_short_deg() {
        assert_eq!(format_dec("+52322"), "+ 05° 23' 22''");
    }

    #[test]
    fn format_dec_decimal_seconds() {
        assert_eq!(format_dec("-052322.5"), "- 05° 23' 22.5''");
    }

    #[test]
    fn format_dec_empty() {
        assert_eq!(format_dec(""), "");
    }
}
