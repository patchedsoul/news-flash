use serde::{Deserialize, Serialize};
use std::default::Default;
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

impl SyncInterval {
    pub fn to_minutes(&self) -> Option<u32> {
        match self {
            SyncInterval::Never => None,
            SyncInterval::QuaterHour => Some(15),
            SyncInterval::HalfHour => Some(30),
            SyncInterval::Hour => Some(60),
            SyncInterval::TwoHour => Some(120),
        }
    }

    pub fn to_seconds(&self) -> Option<u32> {
        self.to_minutes().map(|m| m * 60)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub keep_running_in_background: bool,
    pub sync_every: SyncInterval,
    pub prefer_dark_theme: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        GeneralSettings {
            keep_running_in_background: false,
            sync_every: SyncInterval::QuaterHour,
            prefer_dark_theme: false,
        }
    }
}
