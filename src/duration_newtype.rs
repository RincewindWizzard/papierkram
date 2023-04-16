use std::fmt;
use std::fmt::{Display, Formatter};
use cli_table::{Cell, Color, Style};
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use serde::{de, Deserializer};

/// this is a newtype of chrono::Duration that serializes to an i64 of seconds
#[derive(Debug, Clone)]
pub struct Duration {
    pub chrono_duration: chrono::Duration,
}

impl Duration {
    pub fn of(chrono_duration: chrono::Duration) -> Duration {
        Duration {
            chrono_duration,
        }
    }

    pub fn format_unsigned(&self) -> String {
        let cdur = self.chrono_duration;
        let seconds = cdur.num_seconds().rem_euclid(60).abs();
        let minutes = cdur.num_minutes().rem_euclid(60).abs();
        let hours = cdur.num_hours().abs();
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }

    pub fn format_signed(&self) -> String {
        let cdur = self.chrono_duration;
        let sign = if cdur.num_seconds() < 0 {
            "-"
        } else if cdur.num_seconds() > 0 {
            "+"
        } else {
            " "
        };
        let seconds = cdur.num_seconds().rem_euclid(60).abs();
        let minutes = cdur.num_minutes().rem_euclid(60).abs();
        let hours = cdur.num_hours().abs();
        format!("{sign}{hours:02}:{minutes:02}:{seconds:02}")
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cdur = self.chrono_duration;
        let sign = if cdur.num_seconds() < 0 {
            "-"
        } else if cdur.num_seconds() > 0 {
            "+"
        } else {
            " "
        };
        let seconds = cdur.num_seconds().rem_euclid(60).abs();
        let minutes = cdur.num_minutes().rem_euclid(60).abs();
        let hours = cdur.num_hours().abs();
        f.write_str(&format!("{sign}{hours:02}:{minutes:02}:{seconds:02}"))
    }
}

impl From<chrono::Duration> for Duration {
    fn from(chrono_duration: chrono::Duration) -> Self {
        Duration::of(chrono_duration)
    }
}

impl From<Duration> for chrono::Duration {
    fn from(value: Duration) -> Self {
        value.chrono_duration
    }
}

impl serde::Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        self.chrono_duration.num_seconds().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
    {
        Ok(Duration {
            chrono_duration: chrono::Duration::seconds(deserializer.deserialize_i64(I64Visitor)?),
        })
    }
}


struct I64Visitor;

impl<'de> serde::de::Visitor<'de> for I64Visitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^63 and 2^63")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        Ok(value)
    }
}

impl FromSql for Duration {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let seconds = FromSql::column_result(value)?;
        Ok(Duration::of(chrono::Duration::seconds(seconds)))
    }
}

impl ToSql for Duration {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.chrono_duration.num_seconds()))
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration {
            chrono_duration: chrono::Duration::zero(),
        }
    }
}

