use chrono::{Duration, Local, NaiveDateTime, TimeZone};

pub struct DateUtil;

impl DateUtil {
    pub fn format(naive_utc: &NaiveDateTime) -> String {
        let local_datetime = Local.from_utc_datetime(naive_utc);
        let mut date = format!("{}", local_datetime.format("%e.%m.%Y"));
        let now = Local::now().naive_local();
        let now_date = now.date();
        let naive_local_date = local_datetime.naive_local().date();

        if now_date == naive_local_date {
            date = "Today".to_owned();
        } else if now_date - naive_local_date == Duration::days(1) {
            date = "Yesterday".to_owned();
        }

        format!("{} {}", date, local_datetime.format("%k:%M"))
    }
}
