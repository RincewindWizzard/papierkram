use std::collections::HashMap;
use std::path::PathBuf;
use directories::ProjectDirs;
use log::debug;
use serde_derive::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
pub struct ApplicationConfig {
    pub probes: HashMap<String, Probe>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Probe {
    color: Option<Color>,
    pub command: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
}

impl From<Color> for cli_table::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => {
                cli_table::Color::Black
            }
            Color::Blue => {
                cli_table::Color::Blue
            }
            Color::Green => {
                cli_table::Color::Green
            }
            Color::Red => {
                cli_table::Color::Red
            }
            Color::Cyan => {
                cli_table::Color::Cyan
            }
            Color::Magenta => {
                cli_table::Color::Magenta
            }
            Color::Yellow => {
                cli_table::Color::Yellow
            }
            Color::White => {
                cli_table::Color::White
            }
        }
    }
}


impl ApplicationConfig {
    pub fn project_dirs(&self) -> Option<ProjectDirs> {
        let project_dirs = ProjectDirs::from(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_NAME"))?;
        debug!("{project_dirs:?}");
        Some(project_dirs)
    }

    pub fn database_path(&self) -> Option<PathBuf> {
        Some(self.project_dirs()?.data_dir().join(format!("{}.db", env!("CARGO_PKG_NAME"))))
    }
}

impl Default for ApplicationConfig {
    fn default() -> ApplicationConfig {
        debug!("Created a new configuration from default.");
        toml::from_str(include_str!("default.conf")).expect("Default configuration not parseable!")
    }
}