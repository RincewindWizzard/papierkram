use chrono::{NaiveDate, Utc};
use log::debug;
use rusqlite::{Connection, params, Row};
use crate::config::ApplicationConfig;
use crate::datastore::DataStoreError::{Filesystem, UnexpectedRowCount};
use crate::models::OfficeLocation;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum DataStoreError {
    #[error("Errors from rusqlite")]
    SqliteError(#[from] rusqlite::Error),
    #[error("There was an error concerning the filesystem: {0}")]
    Filesystem(String),
    #[error("unknown data store error")]
    Unknown,
    #[error("Statement affected {0} rows instead of expected {1}")]
    UnexpectedRowCount(usize, usize),
}


pub fn connect_database(config: &ApplicationConfig) -> Result<Connection, DataStoreError> {
    let database_path = config
        .database_path()
        .ok_or(Filesystem("Database path not not present!".to_string()))?;

    let dir_path = database_path
        .parent()
        .ok_or(Filesystem(format!("Could not create database path: {}", database_path.display())))?;

    std::fs::create_dir_all(dir_path)
        .or(Err(Filesystem(format!("Could not create database parent path: {}", database_path.display()))))?;

    let connection = Connection::open(database_path.clone())?;

    // running migrations
    let sql = include_str!("sql/create_tables.sql");
    connection.execute(sql, [])?;

    debug!("Succesfully setup database at {}", database_path.display());
    Ok(connection)
}


pub trait LocationStore {
    fn add_current_location(&self, location: &String) -> Result<(), DataStoreError>;
    fn add_location(&self, office_location: &OfficeLocation) -> Result<(), DataStoreError>;
    fn list_locations(&self) -> Result<Vec<OfficeLocation>, DataStoreError>;
    fn list_entries_by_date(&self, date: NaiveDate) -> Result<Vec<OfficeLocation>, DataStoreError>;
}

impl LocationStore for Connection {
    fn add_current_location(&self, location: &String) -> Result<(), DataStoreError> {
        self.add_location(&OfficeLocation {
            instant: Utc::now(),
            location: location.clone(),
        })
    }

    fn add_location(&self, office_location: &OfficeLocation) -> Result<(), DataStoreError> {
        let count = self.execute(
            "REPLACE INTO office_location (instant, location) VALUES (?, ?);",
            params![office_location.instant, office_location.location])?;

        if count == 1 {
            debug!("Successfully inserted location: {office_location:?} into database");
            Ok(())
        } else {
            Err(UnexpectedRowCount(count, 1))
        }
    }

    fn list_locations(&self) -> Result<Vec<OfficeLocation>, DataStoreError> {
        let mut stmt = self.prepare(
            "SELECT instant, location from office_location;",
        )?;

        let entries: Vec<OfficeLocation> = stmt
            .query_map(params![], office_location_from_row)?
            .map(|x| x.unwrap())
            .collect();

        Ok(entries)
    }

    fn list_entries_by_date(&self, date: NaiveDate) -> Result<Vec<OfficeLocation>, DataStoreError> {
        let mut stmt = self.prepare(
            "select instant, location from office_location GROUP BY DATE(instant), location HAVING DATE(instant) = ?;",
        )?;

        let entries: Vec<OfficeLocation> = stmt
            .query_map(params![date], office_location_from_row)?
            .filter_map(|x| x.ok())
            .collect();

        Ok(entries)
    }
}

fn office_location_from_row(row: &Row) -> rusqlite::Result<OfficeLocation> {
    Ok(OfficeLocation {
        instant: row.get(0)?,
        location: row.get(1)?,
    })
}



