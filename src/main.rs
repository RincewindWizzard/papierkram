//! A useful tool to help you doing less paperwork while working.
//! Currently only auto detection of workplace (remote, at-site) is supported.






use std::process;
use anyhow::Error;
use chrono::{TimeZone};
use clap::Parser;


use log::{error, SetLoggerError, warn};
use rusqlite::Connection;

use crate::args::{Args, EventCommand, TogglCommand};

use crate::config::{ApplicationConfig};
use crate::datastore::DataStore;


mod models;
mod args;
mod config;
mod datastore;
mod dates;
mod toggl;
mod commands;
mod duration_newtype;
mod table_cli_helper;


fn setup_logging(args: &Args) -> Result<(), SetLoggerError> {
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
}

pub trait ErrorHandler<T> {
    fn handle_error(self) -> T;
}

impl<T> ErrorHandler<T> for Result<T, anyhow::Error> {
    fn handle_error(self) -> T {
        match self {
            Ok(value) => { value }
            Err(error) => {
                let rust_backtrace = std::env::var("RUST_BACKTRACE")
                    .ok()
                    .map(|var| var == "1")
                    .unwrap_or(false);

                let rust_lib_backtrace = std::env::var("RUST_LIB_BACKTRACE")
                    .ok()
                    .map(|var| var == "1")
                    .unwrap_or(false);

                if rust_backtrace || rust_lib_backtrace {
                    Err::<T, anyhow::Error>(error).unwrap();
                } else {
                    for cause in error.chain() {
                        println!("{cause}");
                    }
                }
                process::exit(1);
            }
        }
    }
}


/// Main Method
fn main() {
    use crate::args::{Commands};
    let args = Args::parse();
    setup_logging(&args).expect("Failed to setup logging!");
    let mut config: ApplicationConfig = ApplicationConfig::load_config().handle_error();

    // Default operation is to show the timesheet
    let command: &Commands = &args.command.unwrap_or(Commands::Toggl {
        sub_command: TogglCommand::Show {
            compact: false,
            start: None,
            end: None,
        },
    });

    if let Commands::Clear {} = command {
        warn!("Removing old database and creating new.");
        config.database_path().map(std::fs::remove_file);
        return;
    }

    let mut connection = Connection::connect_database(&config).expect("Could not connect to Database!");

    match command {
        Commands::Event { sub_command } => {
            match sub_command {
                EventCommand::Insert { date, event } => {
                    crate::commands::event::execute_add(connection, date, event);
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
            crate::commands::toggl::main(&mut config, sub_command, &mut connection);
        }
        Commands::Config { .. } => {
            let toml = toml::to_string(&config);
            println!("{}", toml.unwrap());
        }
    }
}


