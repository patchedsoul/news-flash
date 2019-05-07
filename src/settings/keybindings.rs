use serde_derive::{Deserialize, Serialize};
use gtk::{Window, GtkWindowExt, ShortcutsWindow};
use glib::{object::IsA};
use crate::util::{BuilderHelper, GtkHandle};
use crate::settings::Settings;

#[derive(Debug, Serialize, Deserialize)]
pub struct Keybindings {
    pub general: KeybindingsGeneral,
    pub article_view: KeybindingsArticleView,
    pub article_list: KeybindingsArticleList,
    pub feed_list: KeybindingsFeedList,
}

impl Keybindings {
    pub fn default() -> Self {
        Keybindings {
            general: KeybindingsGeneral::default(),
            article_view: KeybindingsArticleView::default(),
            article_list: KeybindingsArticleList::default(),
            feed_list: KeybindingsFeedList::default(),
        }
    }
}

//--------------------------------------------
// General
//--------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct KeybindingsGeneral {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub quit: Option<String>,
}

impl KeybindingsGeneral {
    pub fn default() -> Self {
        KeybindingsGeneral {
            refresh: Some("F5".to_owned()),
            search: Some("<ctl>F".to_owned()),
            quit: Some("<ctl>Q".to_owned()),
        }
    }
}

//--------------------------------------------
// Article View
//--------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct KeybindingsArticleView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub scroll_up: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub scroll_down: Option<String>,
}

impl KeybindingsArticleView {
    pub fn default() -> Self {
        KeybindingsArticleView {
            scroll_up: Some("I".to_owned()),
            scroll_down: Some("U".to_owned()),
        }
    }
}

//--------------------------------------------
// Article List
//--------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct KeybindingsArticleList {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub read: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub open: Option<String>,
}

impl KeybindingsArticleList {
    pub fn default() -> Self {
        KeybindingsArticleList {
            next: Some("J".to_owned()),
            prev: Some("K".to_owned()),
            read: Some("R".to_owned()),
            mark: Some("M".to_owned()),
            open: Some("O".to_owned()),
        }
    }
}

//--------------------------------------------
// Feed List
//--------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct KeybindingsFeedList {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub expand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub collapse: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub read: Option<String>,
}

impl KeybindingsFeedList {
    pub fn default() -> Self {
        KeybindingsFeedList {
            next: Some("<ctl>J".to_owned()),
            prev: Some("<ctl>K".to_owned()),
            expand: Some("right".to_owned()),
            collapse: Some("left".to_owned()),
            read: Some("<Shift>A".to_owned()),
        }
    }
}

//--------------------------------------------
// Shortcut window
//--------------------------------------------

pub struct NewsFlashShortcutWindow {
    widget: ShortcutsWindow,
}

impl NewsFlashShortcutWindow {
    pub fn new<D: IsA<Window> + GtkWindowExt>(settings_dialog: &D, settings: &GtkHandle<Settings>) -> Self {
        let builder = BuilderHelper::new("shorcuts_window");

        let widget = builder.get::<ShortcutsWindow>("shortcuts-window");
        widget.set_transient_for(settings_dialog);

        NewsFlashShortcutWindow {
            widget: widget,
        }
    }

    pub fn widget(&self) -> ShortcutsWindow {
        self.widget.clone()
    }
}