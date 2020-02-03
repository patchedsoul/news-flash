use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProxyProtocoll {
    ALL,
    HTTP,
    HTTPS,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyModel {
    pub protocoll: ProxyProtocoll,
    pub url: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub proxy: Vec<ProxyModel>,
    pub accept_invalid_certs: bool,
    pub accept_invalid_hostnames: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        AdvancedSettings {
            proxy: Vec::new(),
            accept_invalid_certs: false,
            accept_invalid_hostnames: false,
        }
    }
}
