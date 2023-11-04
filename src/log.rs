use chrono::naive::serde::ts_nanoseconds;
use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Log {
    #[serde(with = "ts_nanoseconds")]
    pub datetime: NaiveDateTime,
    pub level: String,
    pub message: String,
}

impl Log {
    pub fn parse_from_str(line: &str) -> Log {
        let v: Vec<&str> = line.split_whitespace().collect();

        let datetime_str = format!("{} {}", v[0], v[1]);
        let datetime = NaiveDateTime::parse_from_str(&datetime_str, "%d-%m-%Y %T%.9f").unwrap();

        let level = str::replace(v[2], ":", "");

        let message = v.get(3..).unwrap().join(" ");

        Log {
            datetime,
            level,
            message,
        }
    }
}
