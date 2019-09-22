mod article_list;
mod article_view;
mod dialog;
mod error;
mod general;
mod keybinding_editor;
mod keybindings;
mod theme_chooser;

use self::error::{SettingsError, SettingsErrorKind};
use crate::article_view::ArticleTheme;
use crate::main_window::DATA_DIR;
use article_list::ArticleListSettings;
use article_view::ArticleViewSettings;
pub use dialog::SettingsDialog;
use failure::ResultExt;
use general::GeneralSettings;
pub use keybindings::{Keybindings, NewsFlashShortcutWindow};
use news_flash::models::ArticleOrder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

static CONFIG_NAME: &'static str = "newflash_gtk.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    general: GeneralSettings,
    article_list: ArticleListSettings,
    article_view: ArticleViewSettings,
    keybindings: Keybindings,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    path: PathBuf,
}

impl Settings {
    pub fn open() -> Result<Self, SettingsError> {
        let path = DATA_DIR.join(CONFIG_NAME);
        if path.as_path().exists() {
            let data = fs::read_to_string(&path).context(SettingsErrorKind::ReadFromDisk)?;
            let mut settings: Self = serde_json::from_str(&data).context(SettingsErrorKind::InvalidJsonContent)?;
            settings.path = path.clone();
            return Ok(settings);
        }

        fs::create_dir_all(DATA_DIR.as_path()).context(SettingsErrorKind::CreateDirectory)?;

        let settings = Settings {
            general: GeneralSettings::default(),
            article_list: ArticleListSettings::default(),
            article_view: ArticleViewSettings::default(),
            keybindings: Keybindings::default(),
            path: path.clone(),
        };
        settings.write().context(SettingsErrorKind::WriteToDisk)?;
        Ok(settings)
    }

    fn write(&self) -> Result<(), SettingsError> {
        let data = serde_json::to_string_pretty(self).context(SettingsErrorKind::Serialize)?;
        fs::write(&self.path, data).context(SettingsErrorKind::WriteToDisk)?;
        Ok(())
    }

    pub fn get_keep_running_in_background(&self) -> bool {
        self.general.keep_running_in_background
    }

    pub fn set_keep_running_in_background(&mut self, keep_running: bool) -> Result<(), SettingsError> {
        self.general.keep_running_in_background = keep_running;
        self.write()?;
        Ok(())
    }

    pub fn get_prefer_dark_theme(&self) -> bool {
        self.general.prefer_dark_theme
    }

    pub fn set_prefer_dark_theme(&mut self, prefer_dark_theme: bool) -> Result<(), SettingsError> {
        self.general.prefer_dark_theme = prefer_dark_theme;
        self.write()?;
        Ok(())
    }

    pub fn get_article_list_order(&self) -> ArticleOrder {
        self.article_list.order.clone()
    }

    pub fn set_article_list_order(&mut self, order: ArticleOrder) -> Result<(), SettingsError> {
        self.article_list.order = order;
        self.write()?;
        Ok(())
    }

    pub fn get_article_view_theme(&self) -> ArticleTheme {
        self.article_view.theme.clone()
    }

    pub fn set_article_view_theme(&mut self, theme: ArticleTheme) -> Result<(), SettingsError> {
        self.article_view.theme = theme;
        self.write()?;
        Ok(())
    }

    pub fn get_article_view_allow_select(&self) -> bool {
        self.article_view.allow_select
    }

    pub fn set_article_view_allow_select(&mut self, allow: bool) -> Result<(), SettingsError> {
        self.article_view.allow_select = allow;
        self.write()?;
        Ok(())
    }

    pub fn get_article_view_font(&self) -> Option<String> {
        self.article_view.font.clone()
    }

    pub fn set_article_view_font(&mut self, font: Option<String>) -> Result<(), SettingsError> {
        self.article_view.font = font;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_shortcut(&self) -> Option<String> {
        self.keybindings.general.shortcut.clone()
    }

    pub fn set_keybind_shortcut(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.general.shortcut = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_refresh(&self) -> Option<String> {
        self.keybindings.general.refresh.clone()
    }

    pub fn set_keybind_refresh(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.general.refresh = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_search(&self) -> Option<String> {
        self.keybindings.general.search.clone()
    }

    pub fn set_keybind_search(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.general.search = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_quit(&self) -> Option<String> {
        self.keybindings.general.quit.clone()
    }

    pub fn set_keybind_quit(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.general.quit = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_all_articles(&self) -> Option<String> {
        self.keybindings.general.all_articles.clone()
    }

    pub fn set_keybind_all_articles(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.general.all_articles = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_only_unread(&self) -> Option<String> {
        self.keybindings.general.only_unread.clone()
    }

    pub fn set_keybind_only_unread(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.general.only_unread = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_only_starred(&self) -> Option<String> {
        self.keybindings.general.only_starred.clone()
    }

    pub fn set_keybind_only_starred(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.general.only_starred = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_view_up(&self) -> Option<String> {
        self.keybindings.article_view.scroll_up.clone()
    }

    pub fn set_keybind_article_view_up(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.article_view.scroll_up = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_view_down(&self) -> Option<String> {
        self.keybindings.article_view.scroll_down.clone()
    }

    pub fn set_keybind_article_view_down(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.article_view.scroll_down = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_next(&self) -> Option<String> {
        self.keybindings.article_list.next.clone()
    }

    pub fn set_keybind_article_list_next(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.article_list.next = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_prev(&self) -> Option<String> {
        self.keybindings.article_list.prev.clone()
    }

    pub fn set_keybind_article_list_prev(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.article_list.prev = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_read(&self) -> Option<String> {
        self.keybindings.article_list.read.clone()
    }

    pub fn set_keybind_article_list_read(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.article_list.read = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_mark(&self) -> Option<String> {
        self.keybindings.article_list.mark.clone()
    }

    pub fn set_keybind_article_list_mark(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.article_list.mark = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_article_list_open(&self) -> Option<String> {
        self.keybindings.article_list.open.clone()
    }

    pub fn set_keybind_article_list_open(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.article_list.open = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_next(&self) -> Option<String> {
        self.keybindings.feed_list.next.clone()
    }

    pub fn set_keybind_feed_list_next(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.feed_list.next = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_prev(&self) -> Option<String> {
        self.keybindings.feed_list.prev.clone()
    }

    pub fn set_keybind_feed_list_prev(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.feed_list.prev = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_feed_list_toggle_expanded(&self) -> Option<String> {
        self.keybindings.feed_list.toggle_expanded.clone()
    }

    pub fn set_keybind_feed_list_toggle_expanded(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.feed_list.toggle_expanded = key;
        self.write()?;
        Ok(())
    }

    pub fn get_keybind_sidebar_set_read(&self) -> Option<String> {
        self.keybindings.feed_list.read.clone()
    }

    pub fn set_keybind_sidebar_set_read(&mut self, key: Option<String>) -> Result<(), SettingsError> {
        self.keybindings.feed_list.read = key;
        self.write()?;
        Ok(())
    }
}
