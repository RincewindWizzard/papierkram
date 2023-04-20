use thiserror::Error;
use std::collections::HashMap;


use chrono::{Duration, NaiveDate, Utc};
use log::debug;
use rusqlite::{Connection, Params, params, Row};

use crate::config::ApplicationConfig;
use crate::datastore::DataStoreError::FileSystem;


use crate::models::{Event, ExpectedDuration, TimeEntry, TimeSheet};

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[derive(Error, Debug)]
enum DataStoreError {
    #[error("Cannot acces database file: {0}")]
    FileSystem(String)
}

pub trait DataStore {
    fn connect_database(config: &ApplicationConfig) -> Result<Connection>;

    fn run_migrations(&mut self) -> Result<()>;

    /// Executes a select statement and converts all Rows into T using the function from_row
    fn view_query<T, F, P>(&mut self, sql: &str, params: P, fro_row: F) -> Result<Vec<T>>
        where
            F: FnMut(&Row<'_>) -> rusqlite::Result<T, rusqlite::Error>,
            P: Params;

    /// Inserts a Vector of Documents into a table using the sql statement and the function to_row
    fn insert_query<'a, T, F, P>(&mut self, sql: &str, docs: &'a Vec<T>, to_row: F) -> Result<()>
        where
            F: Fn(&'a T) -> P,
            P: Params;


    fn insert_event(&mut self, event: &Event) -> Result<()>;
    fn insert_current_event(&mut self, name: &String) -> Result<()>;
    fn insert_events(&mut self, events: &Vec<Event>) -> Result<()>;
    fn insert_time_entry(&mut self, time_entry: &TimeEntry) -> Result<()>;
    fn insert_time_entries(&mut self, time_entry: &Vec<TimeEntry>) -> Result<()>;
    fn insert_expected_duration(&mut self, expected_duration: ExpectedDuration) -> Result<()>;

    /// inserts the default expected duration for every date where a time entry exists
    /// but no expected duration ist given.
    fn insert_default_expected_duration(&mut self, default: Duration) -> Result<()>;

    /// lists all events sorted by date ASC
    fn list_events(&mut self) -> Result<Vec<Event>>;
    /// lists all time entries sorted by date ASC
    fn list_time_entries(&mut self) -> Result<Vec<TimeEntry>>;

    /// lists all expected durations sorted by date ASC
    fn list_expected_durations(&mut self) -> Result<Vec<ExpectedDuration>>;

    /// returns a map of all expected durations per date
    fn view_expected_durations_by_date(&mut self) -> Result<HashMap<NaiveDate, Duration>>;

    /// returns a map of all events grouped by their dates
    fn view_event_by_date(&mut self) -> Result<HashMap<NaiveDate, Vec<Event>>>;

    fn view_events_where_date_eq(&mut self, date: NaiveDate) -> Result<Vec<Event>>;

    // view all known event names
    fn view_event_names(&mut self) -> Result<Vec<String>>;

    /// returns the timesheet with all necessary information
    fn view_timesheet(&mut self, start: NaiveDate, end: NaiveDate) -> Result<TimeSheet>;

    /// returns the timesheet with all necessary information
    /// includes weekends and holidays
    fn view_full_timesheet(&mut self, start: NaiveDate, end: NaiveDate) -> Result<TimeSheet>;


    fn view_timesheet_export(&mut self) -> Result<TimeSheet>;
}


impl DataStore for Connection {
    fn connect_database(config: &ApplicationConfig) -> Result<Connection> {
        let database_path = config
            .database_path()
            .ok_or(FileSystem("Database path not not present!".to_string()))?;

        let dir_path = database_path
            .parent()
            .ok_or(FileSystem(format!("Could not create database path: {}", database_path.display())))?;

        std::fs::create_dir_all(dir_path)?;

        let mut connection = Connection::open(database_path.clone())?;
        connection.run_migrations()?;

        debug!("Succesfully setup database at {}", database_path.display());

        Ok(connection)
    }

    fn run_migrations(&mut self) -> Result<()> {
        // running migrations
        let sql = include_str!("sql/create_tables.sql");
        debug!("Executing sql: {sql}");

        Ok(self.execute_batch(sql)?)
    }


    fn view_query<T, F, P>(&mut self, sql: &str, params: P, from_row: F) -> Result<Vec<T>>
        where
            F: FnMut(&Row<'_>) -> rusqlite::Result<T, rusqlite::Error>,
            P: Params
    {
        let tx = self.transaction()?;

        let result = tx
            .prepare(sql)?
            .query_map(params, from_row)?
            .filter_map(|x| x.ok())
            .collect();

        tx.commit()?;
        Ok(result)
    }

    fn insert_query<'a, T, F, P>(&mut self, sql: &str, docs: &'a Vec<T>, to_row: F) -> Result<()>
        where
            F: Fn(&'a T) -> P,
            P: Params
    {
        let tx = self.transaction()?;
        {
            let mut stmt = tx.prepare(sql)?;

            let _errors: Vec<rusqlite::Error> = docs
                .iter()
                .map(to_row)
                .map(|params| stmt.execute(params))
                .filter_map(|result| result.err())
                .collect();
        }
        tx.commit()?;
        Ok(())
    }

    fn insert_event(&mut self, event: &Event) -> Result<()> {
        self.insert_events(&vec![event.clone()])
    }

    fn insert_current_event(&mut self, name: &String) -> Result<()> {
        self.insert_event(&Event {
            time: Utc::now(),
            name: name.clone(),
        })
    }

    fn insert_events(&mut self, events: &Vec<Event>) -> Result<()> {
        self.insert_query(
            "REPLACE INTO office_location (instant, location) VALUES (?, ?);",
            events,
            |event| (
                event.time,
                event.name.clone()
            ),
        )
    }

    fn insert_time_entry(&mut self, time_entry: &TimeEntry) -> Result<()> {
        self.insert_time_entries(&vec![time_entry.clone()])
    }

    fn insert_time_entries(&mut self, time_entries: &Vec<TimeEntry>) -> Result<()> {
        self.insert_query(
            "REPLACE INTO time_entries (id, description, start, stop, project_id, workspace_id) VALUES (?, ?, ?, ?, ?, ?);",
            time_entries,
            |time_entry| (
                time_entry.id,
                time_entry.description.clone(),
                time_entry.start,
                time_entry.stop,
                time_entry.project_id,
                time_entry.workspace_id
            ),
        )
    }

    fn insert_expected_duration(&mut self, expected_duration: ExpectedDuration) -> Result<()> {
        self.insert_query(
            "REPLACE INTO expected_duration (date, duration) VALUES (?, ?);",
            &vec![expected_duration],
            |expected_duration| (
                expected_duration.date,
                expected_duration.duration.clone(),
            ),
        )
    }

    fn insert_default_expected_duration(&mut self, default: Duration) -> Result<()> {
        self.insert_query(
            "INSERT OR IGNORE into expected_duration select DATE(start) as date, ? as duration FROM time_entries GROUP BY DATE(start);",
            &vec![default],
            |duration| (
                duration.num_seconds(),
            ),
        )
    }

    fn list_events(&mut self) -> Result<Vec<Event>> {
        self.view_query(
            "SELECT instant, location from office_location;",
            params![],
            |row| Ok(crate::models::Event {
                time: row.get("instant")?,
                name: row.get("location")?,
            }),
        )
    }

    fn list_time_entries(&mut self) -> Result<Vec<TimeEntry>> {
        self.view_query(
            "SELECT id, description, start, stop, project_id, workspace_id from time_entries;",
            params![],
            |row| Ok(crate::models::TimeEntry {
                id: row.get("id")?,
                description: row.get("description")?,
                start: row.get("start")?,
                stop: row.get("stop")?,
                project_id: row.get("project_id")?,
                workspace_id: row.get("workspace_id")?,
            }),
        )
    }

    fn list_expected_durations(&mut self) -> Result<Vec<ExpectedDuration>> {
        self.view_query(
            "SELECT date, duration from expected_durations;",
            params![],
            |row| Ok(crate::models::ExpectedDuration {
                date: row.get("date")?,
                duration: row.get("duration")?,
            }),
        )
    }

    fn view_expected_durations_by_date(&mut self) -> Result<HashMap<NaiveDate, Duration>> {
        todo!()
    }

    fn view_event_by_date(&mut self) -> Result<HashMap<NaiveDate, Vec<Event>>> {
        let events = self.list_events()?;
        let mut map = HashMap::new();
        for event in events {
            let date = event.time.date_naive();
            map.entry(date).or_insert(vec![event]);
        }
        Ok(map)
    }

    fn view_events_where_date_eq(&mut self, date: NaiveDate) -> Result<Vec<Event>> {
        self.view_query(
            "select instant, location from office_location GROUP BY DATE(instant), location HAVING DATE(instant) = ?;",
            params![date],
            |row| Ok(crate::models::Event {
                time: row.get("instant")?,
                name: row.get("location")?,
            }),
        )
    }

    fn view_event_names(&mut self) -> Result<Vec<String>> {
        self.view_query(
            "select DISTINCT(location) as name from office_location;",
            params![],
            |row| row.get("name"),
        )
    }

    fn view_timesheet(&mut self, start: NaiveDate, end: NaiveDate) -> Result<TimeSheet> {
        debug!("Loading timesheet from {} to {}.", start, end);
        let timesheet = self.view_query(
            include_str!("sql/select_report.sql"),
            params![start, end],
            |row| Ok(crate::models::TimeSheetRow {
                date: row.get("date")?,
                actual_duration: row.get("actual_duration")?,
                expected_duration: row.get("expected_duration")?,
                delta: row.get("delta")?,
                saldo: row.get("saldo")?,
                normalized_start_of_business: row.get("normalized_start_of_business")?,
                normalized_end_of_business: row.get("normalized_end_of_business")?,
                locations: row.get("events").unwrap_or("".to_string()),
            }),
        )?;

        Ok(timesheet)
    }

    fn view_full_timesheet(&mut self, start: NaiveDate, end: NaiveDate) -> Result<TimeSheet> {
        let timesheet = self.view_timesheet(start, end)?;

        if !timesheet.is_empty() {
            let start = timesheet[0].date;
            let end = timesheet.last().unwrap().date + Duration::days(1);
            debug!("Printing timesheet from {} to {}.", start, end);

            let mut full_timesheet: TimeSheet = Vec::new();
            let mut index = 0;
            for days in 0..(end - start).num_days() {
                let current_date = start + Duration::days(days);
                if current_date < timesheet[index].date {
                    full_timesheet.push(crate::models::TimeSheetRow::empty(current_date));
                } else {
                    full_timesheet.push(timesheet[index].clone());
                    index += 1;
                }
            }
            Ok(full_timesheet)
        } else {
            Ok(timesheet)
        }
    }

    fn view_timesheet_export(&mut self) -> Result<TimeSheet> {
        let timesheet = self.view_query(
            include_str!("sql/select_report_export.sql"),
            params![],
            |row| Ok(crate::models::TimeSheetRow {
                date: row.get("date")?,
                actual_duration: row.get("actual_duration")?,
                expected_duration: row.get("expected_duration")?,
                delta: row.get("delta")?,
                saldo: row.get("saldo")?,
                normalized_start_of_business: row.get("normalized_start_of_business")?,
                normalized_end_of_business: row.get("normalized_end_of_business")?,
                locations: row.get("events")?,
            }),
        )?;

        Ok(timesheet)
    }
}


#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveDate, Utc};
    use rusqlite::Connection;
    use crate::datastore::{DataStore};
    use crate::models::TimeEntry;

    #[test]
    fn test_format() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection.run_migrations().unwrap();

        let begin = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        for id in 0..10 {
            let day = begin + Duration::days(id);
            let time_entry = TimeEntry {
                id,
                description: None,
                start: day.and_hms_opt(8, 0, 0).unwrap().and_local_timezone(Utc).unwrap(),
                stop: Some(day.and_hms_opt(12, 0, 0).unwrap().and_local_timezone(Utc).unwrap()),
                project_id: None,
                workspace_id: None,
            };
            connection.insert_time_entry(&time_entry).unwrap();
        }
        let end = begin + Duration::days(10);
        connection.insert_default_expected_duration(Duration::seconds(42)).unwrap();
        assert_eq!(10, connection.list_time_entries().unwrap().len());
        assert_eq!(9, connection.view_timesheet(begin, end).unwrap().len());

        println!("{:?}", connection.view_timesheet(begin, end).unwrap());
    }
}