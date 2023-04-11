use chrono::Duration;
use chrono::{DateTime, Local, NaiveDateTime, Utc};


use serde_derive::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub instant: DateTime<Utc>,
    pub location: String,
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

impl TimeEntry {
    pub(crate) fn duration(&self) -> Option<Duration> {
        self.stop.map(|stop| stop - self.start)
    }
}


impl From<&Event> for Vec<cli_table::CellStruct> {
    fn from(office_location: &Event) -> Vec<cli_table::CellStruct> {
        use cli_table::Cell;
        vec![
            office_location.instant.with_timezone(&Local).cell(),
            office_location.location.clone().cell(),
        ]
    }
}





