use super::locale::{FirstDayOfWeek, Locale};
use chrono::{Datelike, Duration, NaiveDate};

pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    match next {
        Some(date) => (date - Duration::days(1)).day(),
        None => 31,
    }
}

pub fn first_weekday_offset(year: i32, month: u32, first_day: FirstDayOfWeek) -> u32 {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let wd = first.weekday();
    match first_day {
        FirstDayOfWeek::Monday => wd.num_days_from_monday(),
        FirstDayOfWeek::Sunday => wd.num_days_from_sunday(),
    }
}

pub fn prev_month_date(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

pub fn next_month_date(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

pub fn format_month_year(year: i32, month: u32, locale: &Locale) -> String {
    format!("{} {}", locale.month_name(month), year)
}

pub fn weekday_headers(locale: &Locale) -> Vec<&'static str> {
    let base = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(); // any Monday
    match locale.first_day_of_week {
        FirstDayOfWeek::Monday => (0..7)
            .map(|i| {
                let date = base + Duration::days(i);
                locale.weekday_short_name(date.weekday())
            })
            .collect(),
        FirstDayOfWeek::Sunday => {
            let sun_base = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();
            (0..7)
                .map(|i| {
                    let date = sun_base + Duration::days(i);
                    locale.weekday_short_name(date.weekday())
                })
                .collect()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DayInfo {
    pub date: NaiveDate,
    pub is_outside: bool,
    pub is_today: bool,
}

pub fn generate_month_days(
    year: i32,
    month: u32,
    today_date: NaiveDate,
    first_day: FirstDayOfWeek,
) -> Vec<DayInfo> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let offset = first_weekday_offset(year, month, first_day);
    let start = first - Duration::days(offset as i64);
    let total_cells = 42;

    (0..total_cells)
        .map(|i| {
            let date = start + Duration::days(i as i64);
            DayInfo {
                date,
                is_outside: date.month() != month,
                is_today: date == today_date,
            }
        })
        .collect()
}
