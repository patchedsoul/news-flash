use serde_derive::{Deserialize, Serialize};
use gtk::{Box, Cast, ContainerExt, Window, GtkWindowExt, ShortcutsWindow, Stack, StackExt, BinExt, WidgetExt};
use glib::{object::IsA};
use crate::util::{BuilderHelper, GtkHandle, GTK_RESOURCE_FILE_ERROR};
use crate::settings::Settings;
use crate::Resources;
use std::str;

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
    pub shortcut: Option<String>,
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
            shortcut: Some("F1".to_owned()),
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
            expand: Some("Right".to_owned()),
            collapse: Some("Left".to_owned()),
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
        let ui_data = Resources::get("ui/shorcuts_window.ui")
            .expect(GTK_RESOURCE_FILE_ERROR);
        let mut ui_xml = str::from_utf8(ui_data.as_ref())
            .expect(GTK_RESOURCE_FILE_ERROR)
            .to_owned();
        
        ui_xml = Self::setup_shortcut(&ui_xml, "$SHORTCUT", settings.borrow().get_keybind_shortcut());
        ui_xml = Self::setup_shortcut(&ui_xml, "$REFRESH", settings.borrow().get_keybind_refresh());
        ui_xml = Self::setup_shortcut(&ui_xml, "$SEARCH", settings.borrow().get_keybind_search());
        ui_xml = Self::setup_shortcut(&ui_xml, "$QUIT", settings.borrow().get_keybind_quit());
        ui_xml = Self::setup_shortcut(&ui_xml, "$NEXTART", settings.borrow().get_keybind_article_list_next());
        ui_xml = Self::setup_shortcut(&ui_xml, "$PREVART", settings.borrow().get_keybind_article_list_prev());
        ui_xml = Self::setup_shortcut(&ui_xml, "$TOGGLEREAD", settings.borrow().get_keybind_article_list_read());
        ui_xml = Self::setup_shortcut(&ui_xml, "$TOGGLEMARKED", settings.borrow().get_keybind_article_list_mark());
        ui_xml = Self::setup_shortcut(&ui_xml, "$OPENBROWSER", settings.borrow().get_keybind_article_list_open());
        ui_xml = Self::setup_shortcut(&ui_xml, "$SCROLLUP", settings.borrow().get_keybind_article_view_up());
        ui_xml = Self::setup_shortcut(&ui_xml, "$SCROLLDOWN", settings.borrow().get_keybind_article_view_down());
        ui_xml = Self::setup_shortcut(&ui_xml, "$NEXTFEED", settings.borrow().get_keybind_feed_list_next());
        ui_xml = Self::setup_shortcut(&ui_xml, "$PREVFEED", settings.borrow().get_keybind_feed_list_prev());
        ui_xml = Self::setup_shortcut(&ui_xml, "$CATCOLLAPSE", settings.borrow().get_keybind_feed_list_collapse());
        ui_xml = Self::setup_shortcut(&ui_xml, "$CATEXPAND", settings.borrow().get_keybind_feed_list_expand());
        ui_xml = Self::setup_shortcut(&ui_xml, "$FEEDREAD", settings.borrow().get_keybind_feed_list_read());
        

        let builder = BuilderHelper::new_from_xml(&ui_xml);
        let widget = builder.get::<ShortcutsWindow>("shortcuts-window");
        widget.set_transient_for(settings_dialog);

        // WORKAROUND: actually show the shortcuts and not "internal-search" view
        widget.show_all();
        if let Some(shorcut_box) = widget.get_child() {
            if let Ok(shorcut_box) = shorcut_box.downcast::<Box>() {
                for widget in shorcut_box.get_children() {
                    if let Ok(stack) = widget.downcast::<Stack>() {
                        stack.set_visible_child_name("news-flash");
                        break;
                    }
                }
            }
        }

        NewsFlashShortcutWindow {
            widget: widget,
        }
    }

    pub fn widget(&self) -> ShortcutsWindow {
        self.widget.clone()
    }

    fn setup_shortcut(xml: &str, needle: &str, shortcut: Option<String>) -> String {
        match shortcut {
            Some(shortcut) => {
                let shortcut = shortcut.replace("&", "&amp;");
                let shortcut = shortcut.replace("<", "&lt;");
                let shortcut = shortcut.replace(">", "&gt;");
                xml.replacen(needle, &shortcut, 1)
            },
            None => xml.replacen(needle, "", 1),
        }
    }
}