use chrono::{NaiveDate, NaiveTime};
use chrono::{DateTime, Local, Utc};
use cli_table::{Cell, format::Justify, Table};


use serde_derive::{Deserialize, Serialize};
use crate::duration_newtype::Duration;
use crate::table_cli_helper::{empty_if_time_null, cell_style_duration_unsigned, cell_style_duration_signed, cell_style_naive_date};

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


#[derive(Table, Debug, Serialize, Deserialize, Clone)]
pub struct TimeSheetRow {
    #[table(title = "Date", justify = "Justify::Left", customize_fn = "cell_style_naive_date")]
    pub date: NaiveDate,

    #[table(title = "Actual", justify = "Justify::Right", customize_fn = "cell_style_duration_unsigned")]
    pub actual_duration: Duration,

    #[table(title = "Expected", justify = "Justify::Right", customize_fn = "cell_style_duration_unsigned")]
    pub expected_duration: Duration,

    #[table(title = "Delta", justify = "Justify::Right", customize_fn = "cell_style_duration_signed")]
    pub delta: Duration,

    #[table(title = "Saldo", justify = "Justify::Right", customize_fn = "cell_style_duration_signed")]
    pub saldo: Duration,

    #[table(title = "SOB", justify = "Justify::Right", customize_fn = "empty_if_time_null")]
    pub normalized_start_of_business: NaiveTime,

    #[table(title = "EOB", justify = "Justify::Right", customize_fn = "empty_if_time_null")]
    pub normalized_end_of_business: NaiveTime,

    #[table(title = "Location", justify = "Justify::Left")]
    pub locations: String,
}


impl TimeSheetRow {
    pub fn empty(date: NaiveDate) -> TimeSheetRow {
        TimeSheetRow {
            date,
            actual_duration: Duration::default(),
            expected_duration: Duration::default(),
            delta: Duration::default(),
            saldo: Duration::default(),
            normalized_start_of_business: Default::default(),
            normalized_end_of_business: Default::default(),
            locations: "".to_string(),
        }
    }
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






