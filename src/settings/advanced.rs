use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub proxy: Option<String>,
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        AdvancedSettings {
            proxy: None,
            accept_invalid_certs: false,
            accept_invalid_hostnames: false,
        }
    }
}