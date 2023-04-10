use chrono::{DateTime, Local, NaiveTime, Utc};

/// Parses a user submitted date string with best effort.
pub fn parse_date_time(date_str: &String) -> anyhow::Result<DateTime<Utc>> {
    let default_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    dateparser::parse_with(date_str, &Local, default_time)
}

