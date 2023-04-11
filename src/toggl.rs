use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use log::debug;
use serde_json::Value;
use crate::config::Toggl;
use crate::models::TimeEntry;
use crate::toggl::TogglApiError::ValueError;

static TOGGL_API_HOST: &str = "api.track.toggl.com";
static TOGGL_TIME_ENTRIES_ENDPOINT: &str = "/api/v9/me/time_entries";

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum TogglApiError {
    #[error("unknown data store error")]
    Unknown,

    #[error("Errors from reqwest")]
    ReqwestError(#[from]  reqwest::Error),

    #[error("Errors from serde")]
    SerdeError(#[from]  serde_json::Error),

    #[error("Error")]
    ValueError(String),

    #[error("Error")]
    ParseError(#[from]  chrono::format::ParseError),

}


pub fn get_time_entries(
    config: &Toggl,
    start_date: &NaiveDate,
    end_date: &NaiveDate)
    -> Result<Vec<TimeEntry>, TogglApiError>
{
    let url = format!("https://{TOGGL_API_HOST}{TOGGL_TIME_ENTRIES_ENDPOINT}?start_date={start_date}&end_date={end_date}");
    log::debug!("Requesting curl -u {}:{} {url}", config.username, config.password);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .basic_auth(
            &config.username,
            Some(&config.password))
        .send()?.text()?;


    let time_entries: Vec<TimeEntry> = serde_json::from_str::<Vec<Value>>(&response)?
        .iter()
        .map(|x| {
            debug!("{:?}", x);
            x
        })
        .map(value_as_time_entry)
        .map(|x| {
            debug!("{:?}", x);
            x
        })
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .collect();


    Ok(time_entries)
}

fn value_as_time_entry(entry: &Value) -> Result<TimeEntry, TogglApiError> {
    let now = Utc::now().naive_utc();
    Ok(TimeEntry {
        id: serde_json::from_value(entry["id"].clone())?,
        description: serde_json::from_value(entry["description"].clone())?,
        start: as_naive_date_time(entry["start"].clone())?.ok_or(ValueError("Start time not set!".to_string()))?,
        stop: as_naive_date_time(entry["stop"].clone())?,
        project_id: serde_json::from_value(entry["project_id"].clone())?,
        workspace_id: serde_json::from_value(entry["workspace_id"].clone())?,
    })
}

fn as_naive_date_time(value: Value) -> Result<Option<DateTime<Utc>>, TogglApiError> {
    if let serde_json::Value::String(date_str) = value {
        let date = DateTime::parse_from_rfc3339(&date_str)?.with_timezone(&Utc);
        Ok(Some(date))
    } else if let serde_json::Value::Null = value {
        Ok(None)
    } else {
        Err(ValueError("Could not parse value!".to_string()))
    }
}