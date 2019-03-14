use chrono::NaiveDateTime;

pub struct DateUtil;

impl DateUtil {
    pub fn format(date: &NaiveDateTime) -> String {
        date.format("%e.%m.%Y %k:%M").to_string()
    }
}