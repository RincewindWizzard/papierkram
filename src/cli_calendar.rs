use std::ops::Rem;
use chrono::{Datelike, Duration, NaiveDate};
use cli_table::{Cell, CellStruct, Style, Table, TableStruct};
use log::debug;


pub fn calendar_table<F>(start: NaiveDate, end: NaiveDate, f: F) -> TableStruct
    where
        F: Fn(NaiveDate) -> CellStruct
{
    debug!("Generating calendar from {start} to {end}");

    let weekdays = vec!["", "Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];

    let header: Vec<CellStruct> = weekdays
        .iter()
        .map(|x| x.cell().bold(true))
        .collect();


    const ROW_SIZE: usize = 8;
    let iso_week_start = start.iso_week().week() as i64;
    let week_count = (end - start).num_weeks().abs() + 1;
    let diff = (end - start).num_days().abs();

    let mut table: Vec<Vec<CellStruct>> = Vec::new();

    for w in 0..week_count {
        let first_day_of_week = start + Duration::weeks(w);
        let mut row = Vec::new();
        row.push(format!("{first_day_of_week} KW{:02}", (w + iso_week_start).rem(52) + 1).cell());
        for _ in 1..ROW_SIZE {
            row.push("".cell())
        }
        table.push(row);
    }

    for i in 0..diff {
        let current_date = start + Duration::days(i);
        let iso_week = (current_date - start).num_weeks() as usize;
        let day_of_week = current_date.weekday().num_days_from_monday() as usize;
        debug!("Adding {current_date} to calendar");
        table[iso_week][day_of_week + 1] = f(current_date);
    }


    table
        .table()
        .title(header)
        .bold(true)
}