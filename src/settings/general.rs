use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SyncInterval {
    Never,
    QuaterHour,
    HalfHour,
    Hour,
    TwoHour,
}

impl fmt::Display for SyncInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            SyncInterval::Never => "Never",
            SyncInterval::QuaterHour => "15 Minutes",
            SyncInterval::HalfHour => "30 Minutes",
            SyncInterval::Hour => "1 Hour",
            SyncInterval::TwoHour => "2 Hours",
        };

        write!(f, "{}", text)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub keep_running_in_background: bool,
    pub sync_every: SyncInterval,
    pub prefer_dark_theme: bool,
}

impl GeneralSettings {
    pub fn default() -> Self {
        GeneralSettings {
            keep_running_in_background: true,
            sync_every: SyncInterval::QuaterHour,
            prefer_dark_theme: false,
        }
    }
}
