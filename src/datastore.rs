use chrono::{NaiveDate, Utc};
use log::debug;
use rusqlite::{Connection, Error, params, Params, Row, Statement};
use crate::config::ApplicationConfig;
use crate::datastore::DataStoreError::{Filesystem, UnexpectedRowCount};
use crate::models::{Event, TimeEntry};

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
    #[error("Multiple")]
    Multiple(Vec<DataStoreError>),
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
    debug!("Executing sql: {sql}");
    connection.execute_batch(sql)?;

    debug!("Succesfully setup database at {}", database_path.display());
    Ok(connection)
}

pub trait SqlConnection {
    fn execute<P: Params>(&self, sql: &str, params: P) -> Result<usize, rusqlite::Error>;
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
    fn replace_into<P: Params>(&self, sql: &str, param: P) -> Result<(), DataStoreError> {
        let count = self.execute(sql, param)?;

        if count == 1 {
            Ok(())
        } else {
            Err(UnexpectedRowCount(count, 1))
        }
    }
}

impl SqlConnection for Connection {
    fn execute<P: Params>(&self, sql: &str, params: P) -> Result<usize, Error> {
        self.execute(sql, params)
    }

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
    fn insert_documents(&mut self, docs: &Vec<T>) -> Result<(), DataStoreError> {
        let errors: Vec<DataStoreError> = docs.iter()
            .map(|doc| self.insert_document(doc))
            .filter(|x| x.is_err())
            .map(|result| result.err().unwrap())
            .collect();

        if errors.len() > 0 {
            Err(DataStoreError::Multiple(errors))
        } else {
            Ok(())
        }
    }
}

impl DocumentStore<Event> for Connection {
    fn list_documents(&self) -> Result<Vec<Event>, DataStoreError> {
        self.select("SELECT instant, location from office_location;", params![])
    }

    fn insert_document(&self, event: &Event) -> Result<(), DataStoreError> {
        self.replace_into(
            "REPLACE INTO office_location (instant, location) VALUES (?, ?);",
            params![event.time, event.name])
    }
}


pub trait EventStore: DocumentStore<Event> {
    fn list_events(&self) -> Result<Vec<Event>, DataStoreError> {
        self.list_documents()
    }
    fn add_event(&self, event: &Event) -> Result<(), DataStoreError> {
        self.insert_document(event)
    }
    fn add_current_event(&self, location: &String) -> Result<(), DataStoreError> {
        self.insert_document(&Event {
            time: Utc::now(),
            name: location.clone(),
        })
    }

    fn list_events_group_by_date(&self, date: NaiveDate) -> Result<Vec<Event>, DataStoreError> {
        self.select(
            "select instant, location from office_location GROUP BY DATE(instant), location HAVING DATE(instant) = ?;",
            params![date],
        )
    }
}

impl<T: DocumentStore<Event>> EventStore for T {}


pub trait FromRow
    where
        Self: Sized,
{
    fn from_row(row: &Row) -> rusqlite::Result<Self>;
}

impl FromRow for Event {
    fn from_row(row: &Row) -> rusqlite::Result<Event> {
        Ok(Event {
            time: row.get("instant")?,
            name: row.get("location")?,
        })
    }
}

impl DocumentStore<TimeEntry> for Connection {
    fn list_documents(&self) -> Result<Vec<TimeEntry>, DataStoreError> {
        self.select("SELECT id, description, start, stop, project_id, workspace_id from time_entries;", params![])
    }

    fn insert_document(&self, time_entry: &TimeEntry) -> Result<(), DataStoreError> {
        self.replace_into(
            "REPLACE INTO time_entries (id, description, start, stop, project_id, workspace_id) VALUES (?, ?, ?, ?, ?, ?);",
            params![
                time_entry.id,
                time_entry.description,
                time_entry.start,
                time_entry.stop,
                time_entry.project_id,
                time_entry.workspace_id
            ])
    }

    fn insert_documents(&mut self, docs: &Vec<TimeEntry>) -> Result<(), DataStoreError> {
        let tx = self.transaction()?;
        let mut stmt = self.prepare("REPLACE INTO time_entries (id, description, start, stop, project_id, workspace_id) VALUES (?, ?, ?, ?, ?, ?);")?;

        let errors: Vec<DataStoreError> = docs.iter()
            .map(|time_entry| stmt.execute(params![
                time_entry.id,
                time_entry.description,
                time_entry.start,
                time_entry.stop,
                time_entry.project_id,
                time_entry.workspace_id
            ]))
            .filter(|x| x.is_err())
            .map(|result| result.err().unwrap())
            .map(|e| DataStoreError::from(e))
            .collect();

        match tx.commit() {
            Ok(_) => { Ok(()) }
            Err(e) => { Err(DataStoreError::SqliteError(e)) }
        }
    }
}

impl FromRow for TimeEntry {
    fn from_row(row: &Row) -> rusqlite::Result<TimeEntry> {
        Ok(TimeEntry {
            id: row.get("id")?,
            description: row.get("description")?,
            start: row.get("start")?,
            stop: row.get("stop")?,
            project_id: row.get("project_id")?,
            workspace_id: row.get("workspace_id")?,
        })
    }
}

