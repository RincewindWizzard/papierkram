use chrono::{DateTime, Local, Utc};


use serde_derive::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OfficeLocation {
    pub instant: DateTime<Utc>,
    pub location: String,
}


impl From<&OfficeLocation> for Vec<cli_table::CellStruct> {
    fn from(office_location: &OfficeLocation) -> Vec<cli_table::CellStruct> {
        use cli_table::Cell;
        vec![
            office_location.instant.with_timezone(&Local).cell(),
            office_location.location.clone().cell(),
        ]
    }
}





