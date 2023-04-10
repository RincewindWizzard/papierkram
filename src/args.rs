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

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// adds a new location to the database
    Add {
        /// manually overwrite the timestamp of the entry
        #[arg(short, long)]
        date: Option<String>,
        location: String,
    },
    /// Prints a calender showing the weeks in rows
    Calendar {
        start: Option<String>,
        end: Option<String>,
    },
    Clear {},
    Detect {},
    Export {},
    Import {},
    List {},
}


