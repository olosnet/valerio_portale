use chrono::{NaiveDate, Datelike, Weekday};

#[derive(Clone, Copy, PartialEq)]
pub enum FirstDayOfWeek {
    Monday,
    Sunday,
}

#[derive(Clone)]
pub struct Locale {
    pub code: &'static str,
    pub months: [&'static str; 12],
    pub months_genitive: [&'static str; 12],
    pub months_short: [&'static str; 12],
    pub weekdays: [&'static str; 7],
    pub weekdays_short: [&'static str; 7],
    pub first_day_of_week: FirstDayOfWeek,
    pub date_format: &'static str,
}

impl Default for Locale {
    fn default() -> Self {
        locale_en()
    }
}

impl Locale {
    pub fn month_name(&self, month: u32) -> &'static str {
        self.months[month as usize - 1]
    }

    pub fn month_name_genitive(&self, month: u32) -> &'static str {
        self.months_genitive[month as usize - 1]
    }

    pub fn weekday_short_name(&self, wd: Weekday) -> &'static str {
        let idx = match self.first_day_of_week {
            FirstDayOfWeek::Monday => wd.num_days_from_monday() as usize,
            FirstDayOfWeek::Sunday => wd.num_days_from_sunday() as usize,
        };
        self.weekdays_short[idx]
    }

    pub fn format_date(&self, date: NaiveDate) -> String {
        let y = date.year();
        let m = date.month();
        let d = date.day();
        let f = self.date_format;
        let mut result = String::new();
        let bytes = f.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 1 < bytes.len() {
                match bytes[i + 1] {
                    b'Y' => result.push_str(&y.to_string()),
                    b'm' => result.push_str(&format!("{:02}", m)),
                    b'd' => result.push_str(&format!("{:02}", d)),
                    b'e' => result.push_str(&d.to_string()),
                    b'B' => result.push_str(self.month_name(m)),
                    b'b' => result.push_str(self.months_short[m as usize - 1]),
                    b'A' => result.push_str(self.weekdays[date.weekday().num_days_from_sunday() as usize]),
                    b'a' => result.push_str(self.weekdays_short[date.weekday().num_days_from_sunday() as usize]),
                    b'G' => result.push_str(self.month_name_genitive(m)),
                    _ => {
                        result.push(bytes[i] as char);
                        result.push(bytes[i + 1] as char);
                    }
                }
                i += 2;
            } else {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
        result
    }
}

pub fn locale_en() -> Locale {
    Locale {
        code: "en-US",
        months: [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ],
        months_genitive: [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ],
        months_short: [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ],
        weekdays: [
            "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
        ],
        weekdays_short: ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
        first_day_of_week: FirstDayOfWeek::Monday,
        date_format: "%B %d, %Y",
    }
}

pub fn locale_it() -> Locale {
    Locale {
        code: "it-IT",
        months: [
            "Gennaio", "Febbraio", "Marzo", "Aprile", "Maggio", "Giugno",
            "Luglio", "Agosto", "Settembre", "Ottobre", "Novembre", "Dicembre",
        ],
        months_genitive: [
            "gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno",
            "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre",
        ],
        months_short: [
            "Gen", "Feb", "Mar", "Apr", "Mag", "Giu",
            "Lug", "Ago", "Set", "Ott", "Nov", "Dic",
        ],
        weekdays: [
            "Domenica", "Lunedì", "Martedì", "Mercoledì", "Giovedì", "Venerdì", "Sabato",
        ],
        weekdays_short: ["Dom", "Lun", "Mar", "Mer", "Gio", "Ven", "Sab"],
        first_day_of_week: FirstDayOfWeek::Monday,
        date_format: "%d %G %Y",
    }
}
