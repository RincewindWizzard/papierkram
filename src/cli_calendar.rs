use std::collections::HashMap;
use std::ops::Rem;
use chrono::{Datelike, Duration, NaiveDate};
use cli_table::{Cell, CellStruct, Style, Table, TableStruct};
use cli_table::format::{Justify, Padding};
use log::{debug, trace};
use crate::models::Event;
use crate::table_cli_helper::TableFormatter;


pub fn calendar_table(start: NaiveDate, end: NaiveDate, events_per_date: HashMap<NaiveDate, Vec<Event>>, column_max_len: usize) -> TableStruct {
    debug!("Generating calendar from {start} to {end}");

    let weekdays = [
        chrono::Weekday::Mon,
        chrono::Weekday::Tue,
        chrono::Weekday::Wed,
        chrono::Weekday::Thu,
        chrono::Weekday::Fri,
        chrono::Weekday::Sat,
        chrono::Weekday::Sun,
    ]
        .iter()
        .map(|d| d.to_string());

    let mut header = vec![
        String::from(""),
    ];
    header.extend(weekdays);

    let padding = Padding::builder()
        .left(5)
        .right(5)
        .build();

    let header: Vec<CellStruct> = header
        .iter()
        .map(|x| {
            x
                .cell()
                .bold(true)
                .justify(Justify::Center)
                .padding(padding)
        })
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
        trace!("Adding {current_date} to calendar");
        let events: Vec<String> = events_per_date
            .get(&current_date)
            .unwrap_or(&vec![])
            .iter()
            .map(|e| e.name.clone())
            .collect();
        table[iso_week][day_of_week + 1] = events.join(", ").cell().justify(Justify::Center);
    }


    table
        .table()
        .title(header)
        .format_table()
}

