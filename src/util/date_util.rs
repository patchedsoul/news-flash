use chrono::{Datelike, NaiveDateTime, Utc};

pub struct DateUtil;

impl DateUtil {
    pub fn format(chrono_date: &NaiveDateTime) -> String {
        let mut date = format!("{}", chrono_date.format("%e.%m.%Y"));
        let now = Utc::now().naive_utc();
        if now.year() == chrono_date.year() {
            if now.month() == chrono_date.month() {
                if now.day() == chrono_date.day() {
                    date = "Today".to_owned();
                } else if now.day() - 1 == chrono_date.day() {
                    date = "Yesterday".to_owned();
                }
            }
        }

        format!("{} {}", date, chrono_date.format("%k:%M"))
    }
}
