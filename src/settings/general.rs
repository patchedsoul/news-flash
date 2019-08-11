use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub keep_running_in_background: bool,
    pub prefer_dark_theme: bool,
}

impl GeneralSettings {
    pub fn default() -> Self {
        GeneralSettings {
            keep_running_in_background: true,
            prefer_dark_theme: false,
        }
    }
}
