use std::collections::HashMap;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rusqlite::{Connection, Params, params, Row, ToSql, Transaction};
use rusqlite::types::FromSql;


use crate::datastore::SqlConnection;
use crate::models::{Event, ExpectedDuration, TimeEntry, TimeSheet};

type Result<T> = anyhow::Result<T, anyhow::Error>;


pub trait DocumentStore<K, V>
    where
        K: ToSql,
        V: ToSql + FromSql
{
    fn put(&mut self, key: K, value: V);
    fn get(&self, key: &K) -> Option<V>;
}

pub trait TransactionSupplier {
    fn new_transaction(&mut self) -> rusqlite::Result<rusqlite::Transaction<'_>>;
}

impl TransactionSupplier for rusqlite::Connection {
    fn new_transaction(&mut self) -> rusqlite::Result<Transaction<'_>> {
        self.transaction()
    }
}


pub trait DataStore
    where
        Self: TransactionSupplier
{
    /// Executes a select statement and converts all Rows into T using the function from_row
    fn view_query<T, F, P>(&mut self, sql: &str, params: P, fro_row: F) -> Result<Vec<T>>
        where
            F: FnMut(&Row<'_>) -> rusqlite::Result<T, rusqlite::Error>,
            P: Params;

    /// Inserts a Vector of Documents into a table using the sql statement and the function to_row
    fn insert_query<T, F, P>(&mut self, sql: &str, docs: Vec<T>, to_row: F) -> Result<()>
        where
            F: Fn(&T) -> P,
            P: Params;


    fn insert_event(&mut self, event: &Event) -> Result<()>;
    fn insert_events(&mut self, events: &Vec<Event>) -> Result<()>;
    fn insert_time_entry(&mut self, time_entry: &TimeEntry) -> Result<()>;
    fn insert_time_entries(&mut self, time_entry: &Vec<TimeEntry>) -> Result<()>;
    fn insert_expected_duration(&mut self, expected_duration: ExpectedDuration) -> Result<()>;

    /// inserts the default expected duration for every date where a time entry exists
    /// but no expected duration ist given.
    fn insert_default_expected_duration(&mut self, default: ExpectedDuration);

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

    /// returns the timesheet with all necessary information
    fn view_timesheet(&mut self) -> Result<TimeSheet>;
}


impl DataStore for Connection {
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

    fn insert_query<T, F, P>(&mut self, sql: &str, docs: Vec<T>, to_row: F) -> Result<()>
        where
            F: Fn(&T) -> P,
            P: Params
    {
        let tx = self.transaction()?;
        {
            let mut stmt = tx.prepare(sql)?;

            let errors: Vec<rusqlite::Error> = docs
                .iter()
                .map(to_row)
                .map(|params| stmt.execute(params))
                .map(|result| result.err())
                .filter(|o| o.is_some())
                .map(|o| o.unwrap())
                .collect();
        }
        tx.commit()?;
        Ok(())
    }

    fn insert_event(&mut self, event: &Event) -> Result<()> {
        self.insert_events(&vec![event.clone()])
    }

    fn insert_events(&mut self, events: &Vec<Event>) -> Result<()> {
        todo!()
    }

    fn insert_time_entry(&mut self, time_entry: &TimeEntry) -> Result<()> {
        self.insert_time_entries(&vec![time_entry.clone()])
    }

    fn insert_time_entries(&mut self, time_entry: &Vec<TimeEntry>) -> Result<()> {
        todo!()
    }

    fn insert_expected_duration(&mut self, expected_duration: ExpectedDuration) -> Result<()> {
        todo!()
    }

    fn insert_default_expected_duration(&mut self, default: ExpectedDuration) {
        todo!()
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
        todo!()
    }

    fn view_timesheet(&mut self) -> Result<TimeSheet> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::collections::HashMap;
    use chrono::{Duration, NaiveDate};
    use crate::generic_datastore::DocumentStore;

    struct FooStore {
        map: HashMap<NaiveDate, u64>,
    }

    impl DocumentStore<NaiveDate, u64> for FooStore {
        fn put(&mut self, key: NaiveDate, value: u64) {
            self.map.insert(key, value);
        }

        fn get(&self, key: &NaiveDate) -> Option<u64> {
            self.map.get(key).cloned()
        }
    }

    #[test]
    fn test_key_value_store() {
        let mut store = FooStore {
            map: HashMap::new(),
        };

        let date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        for i in 1..10 {
            let current_date = date + Duration::days(i);
            store.put(current_date, i as u64);
            let result = store.get(&current_date).unwrap();
            assert_eq!(i as u64, result);
        }
    }
}