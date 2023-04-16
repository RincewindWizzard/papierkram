use std::fmt;
use chrono::serde::ts_seconds;
use chrono::{NaiveDate, NaiveTime};
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use rusqlite::Row;
use serde::{de, Deserializer};


use serde_derive::{Deserialize, Serialize};
use crate::duration_newtype::Duration;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub time: DateTime<Utc>,
    pub name: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpectedDuration {
    pub date: NaiveDate,

    pub duration: Duration,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeEntry {
    pub id: i64,
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
    pub project_id: Option<i64>,
    pub workspace_id: Option<i64>,
}

pub(crate) type TimeSheet = Vec<TimeSheetRow>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSheetRow {
    pub date: NaiveDate,

    pub actual_duration: Duration,

    pub expected_duration: Duration,

    pub delta: Duration,

    pub saldo: Duration,
    pub normalized_start_of_business: NaiveTime,
    pub normalized_end_of_business: NaiveTime,
}


impl TimeEntry {
    pub(crate) fn duration(&self) -> Option<Duration> {
        self.stop
            .map(|stop| stop - self.start)
            .map(Duration::of)
    }
}


impl From<&Event> for Vec<cli_table::CellStruct> {
    fn from(office_location: &Event) -> Vec<cli_table::CellStruct> {
        use cli_table::Cell;
        vec![
            office_location.time.with_timezone(&Local).cell(),
            office_location.name.clone().cell(),
        ]
    }
}






