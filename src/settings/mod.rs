mod article_list;
mod article_view;
mod sidebar;

use serde_derive::{Deserialize, Serialize};
use article_list::ArticleListSettings;
use article_view::ArticleViewSettings;
use sidebar::SidebarSettings;
use failure::Error;
use crate::main_window::DATA_DIR;
use std::fs;
use std::path::PathBuf;

static CONFIG_NAME: &'static str = "newflash_gtk.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub article_list: ArticleListSettings,
    pub article_view: ArticleViewSettings,
    pub sidebar: SidebarSettings,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    path: PathBuf,
}

impl Settings {
    pub fn open() -> Result<Self, Error> {
        let path = DATA_DIR.join(CONFIG_NAME);

        if path.as_path().exists() {
            let data = fs::read_to_string(&path)?;
            let mut settings: Self = serde_json::from_str(&data)?;
            settings.path = path.clone();
            return Ok(settings);
        }

        let settings = Settings {
            article_list: ArticleListSettings::default(),
            article_view: ArticleViewSettings::default(),
            sidebar: SidebarSettings::default(),
            path: path.clone(),
        };
        settings.write()?;
        Ok(settings)
    }

    fn write(&self) -> Result<(), Error> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&self.path, data)?;
        Ok(())
    }
}