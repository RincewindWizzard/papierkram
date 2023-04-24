
use chrono::{Duration, NaiveDate, Utc};
use cli_table::{WithTitle};
use cli_table::format::{VerticalLine};
use log::{debug, error};
use rusqlite::Connection;
use crate::args::{TogglCommand};
use crate::config;
use crate::config::{ApplicationConfig};
use crate::datastore::DataStore;
use crate::dates::parse_time_interval;


use crate::toggl::get_time_entries;
use crate::table_cli_helper::TableFormatter;

pub fn main(config: &mut ApplicationConfig, command: &crate::args::TogglCommand, connection: &mut Connection) {
    match command {
        TogglCommand::Token { token } => {
            crate::commands::toggl::execute_token(config, token);
        }
        TogglCommand::Show { compact, start, end } => {
            match &config.toggl {
                None => {
                    error!("There is no toggl access configured!")
                }
                Some(_toggl) => {
                    let (start, end) = parse_time_interval(start, end);
                    execute_show(config, connection, *compact, start.date_naive(), end.date_naive());
                }
            }
        }
        TogglCommand::Export { .. } => {
            let timesheet = connection.view_timesheet_export().unwrap();
            let json = serde_json::to_string_pretty(&timesheet).expect("Could not serialize to json!");
            println!("{}", json);
        }
    }
}

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

pub fn execute_show(
    config: &ApplicationConfig,
    connection: &mut Connection,
    compact: bool,
    show_start: NaiveDate,
    show_stop: NaiveDate)
{
    let toggl = config.toggl.as_ref().unwrap();
    let now = Utc::now().date_naive();
    let start = now - Duration::weeks(9);
    let end = now + Duration::days(1);
    let result = get_time_entries(toggl, &start, &end)
        .expect("Could not access the toggl API!");

    debug!("Got all time entries!");

    connection.insert_time_entries(&result)
        .expect("Could not save time entry!");

    debug!("Saved all time entries!");
    let default_expected = Duration::seconds(config.workweek.default_expected_duration_seconds as i64);
    connection.insert_default_expected_duration(default_expected).unwrap();

    let timesheet = if compact {
        connection.view_timesheet(show_start, show_stop)
    } else {
        connection.view_full_timesheet(show_start, show_stop)
    }.unwrap();

    let _vertical_line = VerticalLine::new('│');


    let table = timesheet.with_title().format_table();

    assert!(cli_table::print_stdout(table).is_ok());
}

