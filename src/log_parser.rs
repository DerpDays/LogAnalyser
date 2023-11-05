use chrono::naive::serde::ts_nanoseconds;
use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};

use regex::Regex;


#[derive(Deserialize, Serialize)]
pub struct Log {
    #[serde(with = "ts_nanoseconds")]
    pub datetime: NaiveDateTime,
    pub level: String,
    pub message: String,
}

impl Log {
    pub fn parse_lines(lines: Vec<String>) -> Vec<Log> {
        let mut logs: Vec<Log> = vec![];
        let re = Regex::new(r"(?P<datetime_str>\d{2}-\d{2}-\d{4} \d{2}:\d{2}:\d{2}.\d{9})? ?(?P<level>[A-Z]{3,5})?(: )?(?P<message>.+)?").unwrap();

        for line in lines.iter() {
            let caps = re.captures(line).unwrap();

            let datetime_str_match = caps.name("datetime_str");
            let level_match = caps.name("level");
            let message_match = caps.name("message");

            match (datetime_str_match, level_match) {
                (Some(datetime_str_match), Some(level_match)) => {
                    let datetime_str = datetime_str_match.as_str();
                    let datetime =
                        NaiveDateTime::parse_from_str(datetime_str, "%d-%m-%Y %T%.9f").unwrap();

                    let level = level_match.as_str().to_string();

                    let message: String;
                    if let Some(message_match) = message_match {
                        message = message_match.as_str().to_string();
                    } else {
                        message = String::new();
                    };

                    logs.push(Log {
                        datetime,
                        level,
                        message,
                    })
                }
                (None, None) => match (logs.last_mut(), message_match) {
                    (Some(last_log), Some(message_match)) => {
                        last_log.message += message_match.as_str();
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        logs
    }
}
