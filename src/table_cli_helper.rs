use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Weekday};
use cli_table::{Cell, Color, Style, TableStruct};
use cli_table::format::{Border, HorizontalLine, Separator, VerticalLine};
use crate::duration_newtype::Duration;

pub fn cell_style_naive_date(cell: cli_table::CellStruct, value: &NaiveDate) -> cli_table::CellStruct {
    let color = match value.weekday() {
        Weekday::Mon => {
            None
        }
        Weekday::Tue => {
            None
        }
        Weekday::Wed => {
            None
        }
        Weekday::Thu => {
            None
        }
        Weekday::Fri => {
            None
        }
        Weekday::Sat => {
            Some(Color::Yellow)
        }
        Weekday::Sun => {
            Some(Color::Yellow)
        }
    };

    if value.day0() == 0 {
        cell.bold(true)
    } else {
        cell
    }.foreground_color(color)
}


pub fn cell_style_duration_unsigned(cell: cli_table::CellStruct, value: &Duration) -> cli_table::CellStruct {
    if value.chrono_duration.num_seconds() != 0 {
        value.format_unsigned().cell()
    } else {
        "".cell()
    }
}


pub fn cell_style_duration_signed(cell: cli_table::CellStruct, value: &Duration) -> cli_table::CellStruct {
    if value.chrono_duration.num_seconds() > 0 {
        value.format_signed().cell().foreground_color(Some(Color::Green))
    } else if value.chrono_duration.num_seconds() < 0 {
        value.format_signed().cell().foreground_color(Some(Color::Red))
    } else {
        "".cell()
    }
}

pub fn empty_if_duration_null(cell: cli_table::CellStruct, value: &Duration) -> cli_table::CellStruct {
    if value.chrono_duration.num_seconds() > 0 {
        cell
    } else {
        "".cell()
    }
}

pub fn empty_if_time_null(cell: cli_table::CellStruct, value: &NaiveTime) -> cli_table::CellStruct {
    if value.num_seconds_from_midnight() > 0 {
        cell
    } else {
        "".cell()
    }
}

pub trait TableFormatter {
    fn format_table(self) -> TableStruct;
}

impl TableFormatter for TableStruct {
    fn format_table(self) -> TableStruct
    {
        self
            .border(
                Border::builder()
                    .bottom(HorizontalLine::new('└', '┘', '┴', '─'))
                    .top(HorizontalLine::new('┌', '┐', '┬', '─'))
                    .left(VerticalLine::new('│'))
                    .right(VerticalLine::new('│'))
                    .build()
            )
            .separator(
                Separator::builder()
                    .row(None)
                    .column(Some(VerticalLine::new('│')))
                    .title(Some(HorizontalLine::new('├', '┤', '┼', '─')))
                    .build()
            )
    }
}
