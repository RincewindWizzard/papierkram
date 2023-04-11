use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};
use confy::ConfyError;
use directories::ProjectDirs;
use log::debug;
use serde_derive::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApplicationConfig {
    pub probes: HashMap<String, Probe>,
    pub toggl: Option<Toggl>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Toggl {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Probe {
    pub color: Option<Color>,
    pub command: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
    /// In debug mode create project data in local path
    #[cfg(debug_assertions)]
    pub fn project_dirs() -> Option<ProjectDirs> {
        let exe_path = std::env::current_exe().expect("Could not get the path of the current executable!");
        let mut data_path = PathBuf::from(exe_path.parent()?);
        data_path.push("data");

        debug!("Create path: {}", data_path.display());
        fs::create_dir_all(&data_path).expect("Data Path can not be created!");

        let data_path = fs::canonicalize(&data_path).unwrap_or_else(|_| panic!("Data Path '{}' does not exist!", data_path.display()));


        let project_dirs = ProjectDirs::from_path(data_path)?;

        debug!("{project_dirs:?}");
        Some(project_dirs)
    }

    /// In production mode create project data in the correct paths
    #[cfg(not(debug_assertions))]
    pub fn project_dirs() -> Option<ProjectDirs> {
        let project_dirs = ProjectDirs::from(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_NAME"))?;
        debug!("{project_dirs:?}");
        Some(project_dirs)
    }

    pub fn add_probe(&mut self, name: String, command: String) -> Result<(), ConfyError> {
        self.probes.insert(name, Probe {
            color: None,
            command,
        });
        self.save_config()
    }

    pub fn remove_probe(&mut self, name: String) -> Result<(), ConfyError> {
        self.probes.remove(&name);
        self.save_config()
    }

    pub fn database_path(&self) -> Option<PathBuf> {
        Some(ApplicationConfig::project_dirs()?.data_dir().join(format!("{}.db", env!("CARGO_PKG_NAME"))))
    }

    fn config_file_path() -> Result<PathBuf, ConfyError> {
        let project_dirs = ApplicationConfig::project_dirs()
            .ok_or(ConfyError::BadConfigDirectory("could not determine home directory path".to_string()))?;
        let mut config_file_path = PathBuf::from(project_dirs.config_dir());
        config_file_path.push("main.toml");
        Ok(config_file_path)
    }

    pub fn load_config() -> Result<ApplicationConfig, ConfyError> {
        let config: ApplicationConfig = confy::load_path(ApplicationConfig::config_file_path()?)?;
        Ok(config)
    }


    pub fn save_config(&self) -> Result<(), ConfyError> {
        let _project_dirs = ApplicationConfig::project_dirs()
            .ok_or(ConfyError::BadConfigDirectory("could not determine home directory path".to_string()))?;
        confy::store_path(ApplicationConfig::config_file_path()?, self)
    }
}

impl Default for ApplicationConfig {
    fn default() -> ApplicationConfig {
        debug!("Created a new configuration from default.");
        toml::from_str(include_str!("default.conf")).expect("Default configuration not parseable!")
    }
}