use chrono::{Duration, Utc};
use log::{debug, error};
use rusqlite::Connection;
use crate::config;
use crate::config::{ApplicationConfig, Toggl};
use crate::datastore::DataStore;

use crate::models::TimeEntry;
use crate::toggl::get_time_entries;

pub fn execute_token(config: &mut ApplicationConfig, token: &String) {
    // TODO: might override future attributes
    config.toggl = Some(config::Toggl {
        username: token.clone(),
        password: "api_token".to_string(),
    });
    match config.save_config() {
        Ok(_) => {
            println!("Saved api code!");
        }
        Err(_) => {
            error!("Could not save config file!");
        }
    }
}

pub fn execute_show(toggl: &Toggl, connection: &mut Connection) {
    let now = Utc::now().date_naive();
    let start = now - Duration::weeks(9);
    let end = now + Duration::days(1);
    let mut result = get_time_entries(toggl, &start, &end)
        .expect("Could not access the toggl API!");

    debug!("Got all time entries!");

    // TODO: remove this test higher load
    for _i in 0..1000 {
        result.push(result[0].clone());
    }


    connection.insert_time_entries(&result)
        .expect("Could not save time entry!");


    debug!("Saved all time entries!");

    for time_entry in connection.list_time_entries().expect("Cannot list time entries!") {
        let foo: TimeEntry = time_entry;
        println!("{foo:?}");
    }
}

fn time_report(_connection: &Connection) {}

