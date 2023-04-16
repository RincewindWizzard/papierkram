use std::ops::Deref;
use chrono::{Duration, NaiveDate, Utc};
use cli_table::{Cell, CellStruct, Style, Table, TableStruct, WithTitle};
use cli_table::format::{Border, HorizontalLine, Separator, VerticalLine};
use log::{debug, error};
use rusqlite::Connection;
use crate::args::{Args, TogglCommand};
use crate::config;
use crate::config::{ApplicationConfig, Toggl};
use crate::datastore::DataStore;
use crate::dates::parse_time_interval;

use crate::models::{TimeEntry, TimeSheet, TimeSheetRow};
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
                Some(toggl) => {
                    let (start, end) = parse_time_interval(start, end);
                    execute_show(toggl, connection, *compact, start.date_naive(), end.date_naive());
                }
            }
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
    toggl: &crate::config::Toggl,
    connection: &mut Connection,
    compact: bool,
    show_start: NaiveDate,
    show_stop: NaiveDate)
{
    let now = Utc::now().date_naive();
    let start = now - Duration::weeks(9);
    let end = now + Duration::days(1);
    let result = get_time_entries(toggl, &start, &end)
        .expect("Could not access the toggl API!");

    debug!("Got all time entries!");

    connection.insert_time_entries(&result)
        .expect("Could not save time entry!");

    debug!("Saved all time entries!");
    connection.insert_default_expected_duration(Duration::seconds(5));

    let timesheet = if compact {
        connection.view_timesheet(show_start, show_stop)
    } else {
        connection.view_full_timesheet(show_start, show_stop)
    }.unwrap();

    let vertical_line = VerticalLine::new('│');


    let table = timesheet.with_title().format_table();

    assert!(cli_table::print_stdout(table).is_ok());
}


fn time_report(_connection: &Connection) {}

