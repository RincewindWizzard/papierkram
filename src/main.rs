//! A useful tool to help you doing less paperwork while working.
//! Currently only auto detection of workplace (remote, at-site) is supported.

use std::cmp::{max, min};
use std::collections::HashMap;
use std::io;


use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use clap::Parser;
use confy::ConfyError;


use log::{debug, error, SetLoggerError, warn};
use rusqlite::Connection;

use crate::args::{Args, EventCommand, ProbeCommand, TogglCommand};
use crate::cli_calendar::calendar_table;
use crate::config::{ApplicationConfig, Probe, Toggl};
use crate::datastore::DataStore;
use crate::dates::{parse_date_time};
use crate::models::{Event, TimeEntry};
use crate::toggl::get_time_entries;


mod models;
mod args;
mod config;
mod datastore;
mod dates;
mod cli_calendar;
mod toggl;
mod commands;
mod duration_newtype;


fn setup_logging(args: &Args) -> Result<(), SetLoggerError> {
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
}


/// Main Method
fn main() {
    use crate::args::{Commands};
    let args = Args::parse();
    setup_logging(&args).expect("Failed to setup logging!");
    let mut config: ApplicationConfig = ApplicationConfig::load_config().expect("Could not load configuration!");

    if let Commands::Clear {} = &args.command {
        warn!("Removing old database and creating new.");
        config.database_path().map(std::fs::remove_file);
        return;
    }

    let mut connection = Connection::connect_database(&config).expect("Could not connect to Database!");

    match &args.command {
        Commands::Event { sub_command } => {
            match sub_command {
                EventCommand::Insert { date, event } => {
                    crate::commands::event::execute_add(connection, date, event);
                }
                EventCommand::Calendar { start, end } => {
                    crate::commands::event::execute_calendar(connection, start, end);
                }
                EventCommand::List {} => {
                    crate::commands::event::execute_list(connection);
                }
                EventCommand::Export {} => {
                    crate::commands::event::execute_export(config, connection);
                }
                EventCommand::Import {} => {
                    crate::commands::event::execute_import(config, connection);
                }
            }
        }
        Commands::Detect {} => {
            crate::commands::detect::main(config, connection);
        }
        Commands::Probe { sub_command } => {
            crate::commands::probe::main(&mut config, sub_command);
        }
        Commands::Clear { .. } => {}
        Commands::Toggl { sub_command } => {
            match sub_command {
                TogglCommand::Token { token } => {
                    crate::commands::toggl::execute_token(&mut config, token);
                }
                TogglCommand::Show { .. } => {
                    match config.toggl {
                        None => {
                            error!("There is no toggl access configured!")
                        }
                        Some(toggl) => {
                            crate::commands::toggl::execute_show(&toggl, &mut connection);
                        }
                    }
                }
            }
        }
        Commands::Config { .. } => {
            let toml = toml::to_string(&config);
            println!("{}", toml.unwrap());
        }
    }
}


