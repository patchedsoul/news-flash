mod article_list;
mod article_view;
mod dialog;
mod theme_chooser;
mod keybindings;
mod keybinding_editor;

pub use dialog::SettingsDialog;
pub use keybindings::{NewsFlashShortcutWindow, Keybindings};
use serde_derive::{Deserialize, Serialize};
use article_list::ArticleListSettings;
use article_view::ArticleViewSettings;
use failure::Error;
use crate::main_window::DATA_DIR;
use std::fs;
use std::path::PathBuf;
use news_flash::models::ArticleOrder;
use crate::article_view::ArticleTheme;

static CONFIG_NAME: &'static str = "newflash_gtk.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    article_list: ArticleListSettings,
    article_view: ArticleViewSettings,
    keybindings: Keybindings,
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
            keybindings: Keybindings::default(),
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

    pub fn get_article_list_order(&self) -> ArticleOrder {
        self.article_list.order.clone()
    }

    pub fn set_article_list_order(&mut self, order: ArticleOrder) -> Result<(), Error> {
        self.article_list.order = order;
        self.write()?;
        Ok(())
    }

    pub fn get_article_view_theme(&self) -> ArticleTheme {
        self.article_view.theme.clone()
    }

    pub fn set_article_view_theme(&mut self, theme: ArticleTheme) -> Result<(), Error> {
        self.article_view.theme = theme;
        self.write()?;
        Ok(())
    }

    pub fn get_article_view_allow_select(&self) -> bool {
        self.article_view.allow_select
    }

    pub fn set_article_view_allow_select(&mut self, allow: bool) -> Result<(), Error> {
        self.article_view.allow_select = allow;
        self.write()?;
        Ok(())
    }

    pub fn get_article_view_font(&self) -> Option<String> {
        self.article_view.font.clone()
    }

    pub fn set_article_view_font(&mut self, font: Option<String>) -> Result<(), Error> {
        self.article_view.font = font;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_shortcut(&self) -> Option<String> {
        self.keybindings.general.shortcut.clone()
    }

    pub fn set_keybind_shortcut(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.general.shortcut = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_refresh(&self) -> Option<String> {
        self.keybindings.general.refresh.clone()
    }

    pub fn set_keybind_refresh(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.general.refresh = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_search(&self) -> Option<String> {
        self.keybindings.general.search.clone()
    }

    pub fn set_keybind_search(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.general.search = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_quit(&self) -> Option<String> {
        self.keybindings.general.quit.clone()
    }

    pub fn set_keybind_quit(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.general.quit = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_view_up(&self) -> Option<String> {
        self.keybindings.article_view.scroll_up.clone()
    }

    pub fn set_keybind_article_view_up(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.article_view.scroll_up = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_view_down(&self) -> Option<String> {
        self.keybindings.article_view.scroll_down.clone()
    }

    pub fn set_keybind_article_view_down(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.article_view.scroll_down = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_next(&self) -> Option<String> {
        self.keybindings.article_list.next.clone()
    }

    pub fn set_keybind_article_list_next(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.article_list.next = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_prev(&self) -> Option<String> {
        self.keybindings.article_list.prev.clone()
    }

    pub fn set_keybind_article_list_prev(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.article_list.prev = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_read(&self) -> Option<String> {
        self.keybindings.article_list.read.clone()
    }

    pub fn set_keybind_article_list_read(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.article_list.read = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_mark(&self) -> Option<String> {
        self.keybindings.article_list.mark.clone()
    }

    pub fn set_keybind_article_list_mark(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.article_list.mark = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_open(&self) -> Option<String> {
        self.keybindings.article_list.open.clone()
    }

    pub fn set_keybind_article_list_open(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.article_list.open = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_next(&self) -> Option<String> {
        self.keybindings.feed_list.next.clone()
    }

    pub fn set_keybind_feed_list_next(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.feed_list.next = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_prev(&self) -> Option<String> {
        self.keybindings.feed_list.prev.clone()
    }

    pub fn set_keybind_feed_list_prev(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.feed_list.prev = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_expand(&self) -> Option<String> {
        self.keybindings.feed_list.expand.clone()
    }

    pub fn set_keybind_feed_list_expand(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.feed_list.expand = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_collapse(&self) -> Option<String> {
        self.keybindings.feed_list.collapse.clone()
    }

    pub fn set_keybind_feed_list_collapse(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.feed_list.collapse = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_read(&self) -> Option<String> {
        self.keybindings.feed_list.read.clone()
    }

    pub fn set_keybind_feed_list_read(&mut self, key: Option<String>) -> Result<(), Error> {
        self.keybindings.feed_list.read = key;
        self.write()?;
        Ok(())
    }
}