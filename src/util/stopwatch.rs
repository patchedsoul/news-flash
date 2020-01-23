use std::time;
use log::{debug, error};

#[allow(dead_code)]
pub struct StopWatch {
    start_time: time::SystemTime,
}

impl StopWatch {
    #[allow(dead_code)]
    pub fn start() -> Self {
        StopWatch {
            start_time: time::SystemTime::now(),
        }
    }

    #[allow(dead_code)]
    pub fn log(&self, message: &str) {
        if let Ok(duration) = self.start_time.elapsed() {
            debug!("{} - {} ms", message, duration.as_millis());
        } else {
            error!("system time error");
        }
    }


}