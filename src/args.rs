use clap::Parser;
use clap::Subcommand;

/// Does your paperwork
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    #[arg(short, long)]
    pub(crate) quiet: bool,
}

type Event = String;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// changes the probes in the configuration
    Probe {
        #[command(subcommand)]
        sub_command: ProbeCommand,
    },
    /// shows, inserts and removes events
    Event {
        #[command(subcommand)]
        sub_command: EventCommand,
    },
    /// execute all probes and insert all detected events
    Detect {},

    /// removes database
    Clear {},
}

#[derive(Debug, Subcommand)]
pub enum ProbeCommand {
    /// adds a new probe to the configuration
    Add {
        /// name of the probe.
        event: Event,
        /// command to execute for detection of presence
        cmd: String,
    },
    /// removes a probe from the configuration
    Remove {
        /// name of the probe.
        event: Event,
    },
    /// shows all configured probes
    Show {},
}

#[derive(Debug, Subcommand)]
pub enum EventCommand {
    /// export database to json
    Export {},

    /// import database from json
    Import {},

    /// list all events as a table
    List {},

    /// Prints a calender showing the weeks in rows
    Calendar {
        /// begin of the calendar
        start: Option<String>,
        /// end of the calendar.
        /// leave blank for today
        end: Option<String>,
    },
    /// inserts a new event to the database
    Insert {
        /// manually overwrite the timestamp of the entry
        #[arg(short, long)]
        date: Option<String>,
        /// name of the event to be inserted
        event: Event,
    },

}
