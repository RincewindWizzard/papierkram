use chrono::{NaiveDate, Utc};
use log::debug;
use rusqlite::{Connection, params, Params, Row, Statement};
use crate::config::ApplicationConfig;
use crate::datastore::DataStoreError::{Filesystem, UnexpectedRowCount};
use crate::models::Event;

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

pub trait SqlConnection {
    fn prepare(&self, sql: &str) -> Result<rusqlite::Statement<'_>, rusqlite::Error>;
    fn select<R, P>(&self, sql: &str, param: P) -> Result<Vec<R>, DataStoreError>
        where
            P: Params,
            R: FromRow
    {
        let mut stmt = self.prepare(sql)?;

        let result =
            stmt
                .query_map(param, FromRow::from_row)?
                .filter_map(|x| x.ok())
                .collect();
        Ok(result)
    }
}

impl SqlConnection for Connection {
    fn prepare(&self, sql: &str) -> Result<Statement<'_>, rusqlite::Error> {
        self.prepare(sql)
    }
}

pub trait DocumentStore<T>
    where
        Self: SqlConnection
{
    fn list_documents(&self) -> Result<Vec<T>, DataStoreError>;
    fn insert_document(&self, doc: &T) -> Result<(), DataStoreError>;
}

impl DocumentStore<Event> for Connection {
    fn list_documents(&self) -> Result<Vec<Event>, DataStoreError> {
        let mut stmt = self.prepare(
            "SELECT instant, location from office_location;",
        )?;

        let entries: Vec<Event> = stmt
            .query_map(params![], FromRow::from_row)?
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        Ok(entries)
    }

    fn insert_document(&self, event: &Event) -> Result<(), DataStoreError> {
        let count = self.execute(
            "REPLACE INTO office_location (instant, location) VALUES (?, ?);",
            params![event.instant, event.location])?;

        if count == 1 {
            debug!("Successfully inserted location: {event:?} into database");
            Ok(())
        } else {
            Err(UnexpectedRowCount(count, 1))
        }
    }
}


pub trait EventStore: DocumentStore<Event> {
    fn add_current_event(&self, location: &String) -> Result<(), DataStoreError>;
    fn list_events_group_by_date(&self, date: NaiveDate) -> Result<Vec<Event>, DataStoreError>;
    fn list_events(&self) -> Result<Vec<Event>, DataStoreError> {
        self.list_documents()
    }
    fn add_event(&self, event: &Event) -> Result<(), DataStoreError> {
        self.insert_document(event)
    }
}

impl EventStore for Connection {
    fn add_current_event(&self, location: &String) -> Result<(), DataStoreError> {
        self.insert_document(&Event {
            instant: Utc::now(),
            location: location.clone(),
        })
    }

    fn list_events_group_by_date(&self, date: NaiveDate) -> Result<Vec<Event>, DataStoreError> {
        self.select(
            "select instant, location from office_location GROUP BY DATE(instant), location HAVING DATE(instant) = ?;",
            params![date],
        )
    }
}

pub trait FromRow
    where
        Self: Sized,
{
    fn from_row(row: &Row) -> rusqlite::Result<Self>;
}

impl FromRow for Event {
    fn from_row(row: &Row) -> rusqlite::Result<Event> {
        Ok(Event {
            instant: row.get(0)?,
            location: row.get(1)?,
        })
    }
}


