use std::cmp::{max, min};
use std::io;
use chrono::{Datelike, Duration, NaiveTime, TimeZone, Utc};
use cli_table::Cell;
use log::{debug, error, warn};
use rusqlite::Connection;
use crate::cli_calendar::calendar_table;
use crate::config::ApplicationConfig;
use crate::datastore::DataStore;

use crate::dates::{parse_date_time, parse_time_interval};
use crate::models::Event;

pub fn execute_add(mut connection: Connection, date: &Option<String>, location: &String) {
    match date {
        None => {
            connection.insert_current_event(location)
                .expect("Could not add location!");
        }
        Some(date_str) => {
            let date = parse_date_time(date_str);
            match date {
                Err(_) => {
                    error!("Could not parse date! {}", date_str);
                }
                Ok(date) => {
                    let office_location = Event {
                        time: date.with_timezone(&Utc),
                        name: location.clone(),
                    };
                    connection.insert_event(&office_location)
                        .expect("Could not add location!")
                }
            }
        }
    }
}


pub fn execute_list(mut connection: Connection) {
    use cli_table::{Cell, print_stdout, Style, Table};

    let table = connection.list_events()
        .expect("Could not list locations from database!")
        .iter()
        .map(|x| x.into())
        .collect::<Vec<Vec<cli_table::CellStruct>>>()
        .table()
        .title(vec![
            "Date".cell().bold(true),
            "Location".cell().bold(true),
        ])
        .bold(true);

    assert!(print_stdout(table).is_ok());
}


pub fn execute_export(_config: ApplicationConfig, mut connection: Connection) {
    let rows = connection.list_events().expect("Could not load rows from database!");
    let json = serde_json::to_string_pretty(&rows).expect("Could not serialize to json!");
    println!("{}", json);
}

pub fn execute_import(_config: ApplicationConfig, mut connection: Connection) {
    let rows: Vec<Event> = serde_json::from_reader(io::stdin()).expect("Could not read JSON from stdin!");
    debug!("Read data: {:?}", rows);

    for row in rows {
        connection.insert_event(&row)
            .err()
            .map(|e| warn!("Could not insert row: {:?}; Error: {:?}", row, e));
    }
}


pub fn execute_calendar(mut connection: Connection, start: &Option<String>, end: &Option<String>) {
    use cli_table::Cell;
    let (start, end) = parse_time_interval(start, end);

    let events_per_date = connection.view_event_by_date().unwrap();
    let column_max_len = connection.view_event_names().unwrap().join(", ").len();
    let table = calendar_table(
        start.date_naive(),
        end.date_naive(),
        events_per_date,
        column_max_len,
    );

    assert!(cli_table::print_stdout(table).is_ok());
}






