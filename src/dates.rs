use std::cmp::{max, min};
use chrono::{Datelike, DateTime, Duration, Local, NaiveTime, TimeZone, Utc};

/// Parses a user submitted date string with best effort.
pub fn parse_date_time(date_str: &String) -> anyhow::Result<DateTime<Utc>> {
    let default_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    dateparser::parse_with(date_str, &Local, default_time)
}


pub fn parse_time_interval(start: &Option<String>, end: &Option<String>) -> (DateTime<Utc>, DateTime<Utc>) {
    let end = end.as_ref()
        .map(parse_date_time)
        .and_then(|x| x.ok())
        .unwrap_or(Utc::now() + Duration::days(1));

    let default_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let start_of_year = chrono::NaiveDate::from_ymd_opt(end.year(), 1, 1)
        .map(|x| x.and_time(default_time))
        .and_then(|x| Utc.from_local_datetime(&x).latest())
        .expect("Could not get the start of the year!");


    let start = start.as_ref()
        .map(parse_date_time)
        .and_then(|x| x.ok())
        .unwrap_or(start_of_year);

    let (start, end) = (min(start, end), max(start, end));
    (start, end)
}
