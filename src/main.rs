//! A useful tool to help you doing less paperwork while working.
//! Currently only auto detection of workplace (remote, at-site) is supported.

use std::cmp::{max, min};
use std::io;


use chrono::{Datelike, Duration, NaiveTime, TimeZone, Utc};
use clap::Parser;


use confy::ConfyError;

use log::{debug, error, SetLoggerError, warn};
use rusqlite::Connection;

use crate::args::{Args};
use crate::cli_calendar::calendar_table;
use crate::config::ApplicationConfig;
use crate::datastore::{connect_database, LocationStore};
use crate::dates::{parse_date_time};
use crate::models::OfficeLocation;


mod models;
mod args;
mod config;
mod datastore;
mod dates;
mod cli_calendar;


fn setup_logging(args: &Args) -> Result<(), SetLoggerError> {
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Off)
        .init()
}

fn load_config() -> Result<ApplicationConfig, ConfyError> {
    let config: ApplicationConfig = confy::load(env!("CARGO_PKG_NAME"), "main")?;
    Ok(config)
}


/// Main Method
fn main() {
    use crate::args::{Commands};
    let args = Args::parse();
    setup_logging(&args).expect("Failed to setup logging!");
    let config: ApplicationConfig = load_config().expect("Could not load configuration!");

    if let Commands::Clear {} = &args.command {
        warn!("Removing old database and creating new.");
        config.database_path().map(std::fs::remove_file);
        return;
    }

    let connection = connect_database(&config).expect("Could not connect to Database!");

    match &args.command {
        Commands::Add { date, location } => {
            execute_add(connection, date, location);
        }
        Commands::Calendar { start, end } => {
            execute_calendar(connection, start, end);
        }
        Commands::List {} => {
            execute_list(connection);
        }
        Commands::Export {} => {
            execute_export(config, connection);
        }
        Commands::Import {} => {
            execute_import(config, connection);
        }
        Commands::Detect {} => {
            execute_detect(config, connection);
        }
        _ => {}
    }
}

fn execute_export(_config: ApplicationConfig, connection: Connection) {
    let rows = connection.list_locations().expect("Could not load rows from database!");
    let json = serde_json::to_string_pretty(&rows).expect("Could not serialize to json!");
    println!("{}", json);
}

fn execute_import(_config: ApplicationConfig, connection: Connection) {
    let rows: Vec<OfficeLocation> = serde_json::from_reader(io::stdin()).expect("Could not read JSON from stdin!");
    debug!("Read data: {:?}", rows);

    for row in rows {
        connection.add_location(&row)
            .err()
            .map(|e| warn!("Could not insert row: {:?}; Error: {:?}", row, e));
    }
}

fn execute_detect(config: ApplicationConfig, connection: Connection) {
    use std::process::Command;
    let mut results = Vec::new();
    for (name, probe) in config.probes {
        debug!("Running {name}: {}", probe.command);

        let result = Command::new("sh")
            .arg("-c")
            .arg(probe.command)
            .output()
            .expect("failed to execute process")
            .status
            .success();

        if result {
            results.push(connection.add_current_location(&name));
        } else {
            debug!("{name} was not detected.");
        }
    }

    for result in results {
        result.expect("There was an error while saving the result!");
    }
}


fn execute_calendar(connection: Connection, start: &Option<String>, end: &Option<String>) {
    use cli_table::Cell;
    let end = end.as_ref()
        .map(parse_date_time)
        .and_then(|x| x.ok())
        .unwrap_or(Utc::now() + Duration::days(1));

    let default_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let start_of_year = chrono::NaiveDate::from_ymd_opt(end.year(), 1, 1)
        .map(|x| x.and_time(default_time))
        .and_then(|x| Utc.from_local_datetime(&x).latest())
        .expect("Could not get the start of the year!");


    let start = start.as_ref()
        .map(parse_date_time)
        .and_then(|x| x.ok())
        .unwrap_or(start_of_year);

    let (start, end) = (min(start, end), max(start, end));

    let table = calendar_table(
        start.date_naive(),
        end.date_naive(),
        |date| {
            connection
                .list_entries_by_date(date)
                .unwrap_or(vec![])
                .iter()
                .map(|office_location| office_location.location.clone())
                .collect::<Vec<String>>()
                .join(", ")
                .cell()
        },
    );

    assert!(cli_table::print_stdout(table).is_ok());
}


fn execute_add(connection: Connection, date: &Option<String>, location: &String) {
    match date {
        None => {
            connection.add_current_location(location)
                .expect("Could not add location!");
        }
        Some(date_str) => {
            let date = parse_date_time(date_str);
            match date {
                Err(_) => {
                    error!("Could not parse date! {}", date_str);
                }
                Ok(date) => {
                    let office_location = OfficeLocation {
                        instant: date.with_timezone(&Utc),
                        location: location.clone(),
                    };
                    connection.add_location(&office_location)
                        .expect("Could not add location!")
                }
            }
        }
    }
}


fn execute_list(connection: Connection) {
    use cli_table::{Cell, print_stdout, Style, Table};

    let table = connection.list_locations()
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



