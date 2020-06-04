use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Utc};

pub struct DateUtil;

impl DateUtil {
    pub fn format(naive_utc: &NaiveDateTime) -> String {
        let local_datetime = Local.from_utc_datetime(naive_utc);
        let mut date = format!("{}", local_datetime.format("%e.%m.%Y"));
        let now = Utc::now().naive_utc();
        if now.year() == naive_utc.year() && now.month() == naive_utc.month() {
            if now.day() == naive_utc.day() {
                date = "Today".to_owned();
            } else if now.day() - 1 == naive_utc.day() {
                date = "Yesterday".to_owned();
            }
        }

        format!("{} {}", date, local_datetime.format("%k:%M"))
    }
}
