use super::error::{SettingsError, SettingsErrorKind};
use crate::settings::Settings;
use crate::util::{BuilderHelper, GTK_RESOURCE_FILE_ERROR};
use crate::Resources;
use gdk::{keys::constants, keys::Key, ModifierType};
use glib::object::{Cast, IsA};
use glib::translate::FromGlib;
use gtk::{BinExt, Box, ContainerExt, GtkWindowExt, ShortcutsWindow, Stack, StackExt, WidgetExt, Window};
use log::warn;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::str;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Keybindings {
    pub general: KeybindingsGeneral,
    pub article_view: KeybindingsArticleView,
    pub article_list: KeybindingsArticleList,
    pub feed_list: KeybindingsFeedList,
}

impl Default for Keybindings {
    fn default() -> Self {
        Keybindings {
            general: KeybindingsGeneral::default(),
            article_view: KeybindingsArticleView::default(),
            article_list: KeybindingsArticleList::default(),
            feed_list: KeybindingsFeedList::default(),
        }
    }
}

impl Keybindings {
    pub fn parse_shortcut_string(keybinding: &str) -> Option<String> {
        let (keyval, modifier) = gtk::accelerator_parse(&keybinding);
        Self::parse_shortcut(keyval, modifier)
    }

    pub fn parse_shortcut(keyval: u32, modifier: ModifierType) -> Option<String> {
        let keyval = Self::parse_keyval(keyval);
        let modifier = Self::parse_modifiers(modifier);
        match keyval {
            None => None,
            Some(keyval) => match modifier {
                Some(mut modifier) => {
                    modifier.push_str(&keyval);
                    Some(modifier)
                }
                None => Some(keyval),
            },
        }
    }

    pub fn parse_keyval(keyval: u32) -> Option<String> {
        let keyval = gdk::keyval_to_upper(keyval);
        let keyval = Key::from_glib(keyval);
        let manual_parsed = match keyval {
            constants::Shift_L
            | constants::Control_L
            | constants::Alt_L
            | constants::Meta_L
            | constants::Super_L
            | constants::Hyper_L
            | constants::Shift_R
            | constants::Control_R
            | constants::Alt_R
            | constants::Meta_R
            | constants::Super_R
            | constants::Hyper_R => Self::get_modifier_label(&keyval),
            constants::Left => Some("←".to_owned()),
            constants::Right => Some("→".to_owned()),
            constants::Down => Some("↓".to_owned()),
            constants::Up => Some("↑".to_owned()),
            constants::space => Some("␣".to_owned()),
            constants::Return => Some("⏎".to_owned()),
            constants::Delete => Some("Del".to_owned()),
            constants::Page_Up => Some("⇑".to_owned()),
            constants::Page_Down => Some("⇓".to_owned()),
            constants::BackSpace => Some("Backspace".to_owned()),
            constants::F1 => Some("F1".to_owned()),
            constants::F2 => Some("F2".to_owned()),
            constants::F3 => Some("F3".to_owned()),
            constants::F4 => Some("F4".to_owned()),
            constants::F5 => Some("F5".to_owned()),
            constants::F6 => Some("F6".to_owned()),
            constants::F7 => Some("F7".to_owned()),
            constants::F8 => Some("F8".to_owned()),
            constants::F9 => Some("F9".to_owned()),
            constants::F10 => Some("F10".to_owned()),
            constants::F11 => Some("F11".to_owned()),
            constants::F12 => Some("F12".to_owned()),
            _ => None,
        };

        if manual_parsed.is_some() {
            return manual_parsed;
        }

        match keyval.to_unicode() {
            Some(keyval) => {
                let mut buffer: [u8; 4] = [0; 4];
                Some(keyval.encode_utf8(&mut buffer).to_owned())
            }
            None => None,
        }
    }

    fn get_modifier_label(keyval: &Key) -> Option<String> {
        let mut mod_string = String::new();
        match keyval {
            &constants::Shift_L
            | &constants::Control_L
            | &constants::Alt_L
            | &constants::Meta_L
            | &constants::Super_L
            | &constants::Hyper_L => mod_string.push('L'),
            &constants::Shift_R
            | &constants::Control_R
            | &constants::Alt_R
            | &constants::Meta_R
            | &constants::Super_R
            | &constants::Hyper_R => mod_string.push('R'),
            _ => return None,
        }

        match keyval {
            &constants::Shift_L | &constants::Shift_R => mod_string.push_str("Shift"),
            &constants::Control_L | &constants::Control_R => mod_string.push_str("Ctrl"),
            &constants::Meta_L | &constants::Meta_R => mod_string.push_str("Meta"),
            &constants::Super_L | &constants::Super_R => mod_string.push_str("Super"),
            &constants::Hyper_L | &constants::Hyper_R => mod_string.push_str("Hyper"),
            _ => return None,
        }

        Some(mod_string)
    }

    fn parse_modifiers(modifier: ModifierType) -> Option<String> {
        let mut mod_string = String::new();

        if modifier.contains(ModifierType::SHIFT_MASK) {
            mod_string.push_str("Shift+");
        }
        if modifier.contains(ModifierType::CONTROL_MASK) {
            mod_string.push_str("Ctrl+");
        }
        if modifier.contains(ModifierType::MOD1_MASK) {
            mod_string.push_str("Alt+");
        }
        if modifier.contains(ModifierType::SUPER_MASK) {
            mod_string.push_str("Super+");
        }
        if modifier.contains(ModifierType::HYPER_MASK) {
            mod_string.push_str("Hyper+");
        }
        if modifier.contains(ModifierType::META_MASK) {
            mod_string.push_str("Meta+");
        }

        if mod_string.is_empty() {
            return None;
        }

        Some(mod_string)
    }

    pub fn clean_modifier(mut modifier: ModifierType) -> ModifierType {
        modifier.remove(ModifierType::MOD2_MASK);
        modifier.remove(ModifierType::MOD3_MASK);
        modifier.remove(ModifierType::MOD4_MASK);
        modifier.remove(ModifierType::MOD5_MASK);
        modifier.remove(ModifierType::LOCK_MASK);
        modifier.remove(ModifierType::BUTTON1_MASK);
        modifier.remove(ModifierType::BUTTON2_MASK);
        modifier.remove(ModifierType::BUTTON3_MASK);
        modifier.remove(ModifierType::BUTTON4_MASK);
        modifier.remove(ModifierType::BUTTON5_MASK);
        modifier.remove(ModifierType::RELEASE_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_13_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_14_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_15_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_16_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_17_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_18_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_19_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_20_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_21_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_22_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_23_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_24_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_25_MASK);
        modifier.remove(ModifierType::MODIFIER_RESERVED_29_MASK);

        modifier
    }

    pub fn write_keybinding(
        id: &str,
        keybinding: Option<String>,
        settings: &Arc<RwLock<Settings>>,
    ) -> Result<(), SettingsError> {
        match id {
            "next_article" => settings.write().set_keybind_article_list_next(keybinding),
            "previous_article" => settings.write().set_keybind_article_list_prev(keybinding),
            "toggle_read" => settings.write().set_keybind_article_list_read(keybinding),
            "toggle_marked" => settings.write().set_keybind_article_list_mark(keybinding),
            "open_browser" => settings.write().set_keybind_article_list_open(keybinding),
            "next_item" => settings.write().set_keybind_feed_list_next(keybinding),
            "previous_item" => settings.write().set_keybind_feed_list_prev(keybinding),
            "toggle_category_expanded" => settings.write().set_keybind_feed_list_toggle_expanded(keybinding),
            "sidebar_set_read" => settings.write().set_keybind_sidebar_set_read(keybinding),
            "shortcuts" => settings.write().set_keybind_shortcut(keybinding),
            "refresh" => settings.write().set_keybind_refresh(keybinding),
            "search" => settings.write().set_keybind_search(keybinding),
            "quit" => settings.write().set_keybind_quit(keybinding),
            "all_articles" => settings.write().set_keybind_all_articles(keybinding),
            "only_unread" => settings.write().set_keybind_only_unread(keybinding),
            "only_starred" => settings.write().set_keybind_only_starred(keybinding),
            "scroll_up" => settings.write().set_keybind_article_view_up(keybinding),
            "scroll_down" => settings.write().set_keybind_article_view_down(keybinding),
            "scrap_content" => settings.write().set_keybind_article_view_scrap(keybinding),
            _ => {
                warn!("unexpected keybind id: {}", id);
                Err(SettingsErrorKind::InvalidKeybind.into())
            }
        }
    }

    pub fn read_keybinding(id: &str, settings: &Arc<RwLock<Settings>>) -> Result<Option<String>, SettingsError> {
        match id {
            "next_article" => Ok(settings.read().get_keybind_article_list_next()),
            "previous_article" => Ok(settings.read().get_keybind_article_list_prev()),
            "toggle_read" => Ok(settings.read().get_keybind_article_list_read()),
            "toggle_marked" => Ok(settings.read().get_keybind_article_list_mark()),
            "open_browser" => Ok(settings.read().get_keybind_article_list_open()),
            "next_item" => Ok(settings.read().get_keybind_feed_list_next()),
            "previous_item" => Ok(settings.read().get_keybind_feed_list_prev()),
            "toggle_category_expanded" => Ok(settings.read().get_keybind_feed_list_toggle_expanded()),
            "sidebar_set_read" => Ok(settings.read().get_keybind_sidebar_set_read()),
            "shortcuts" => Ok(settings.read().get_keybind_shortcut()),
            "refresh" => Ok(settings.read().get_keybind_refresh()),
            "search" => Ok(settings.read().get_keybind_search()),
            "quit" => Ok(settings.read().get_keybind_quit()),
            "all_articles" => Ok(settings.read().get_keybind_all_articles()),
            "only_unread" => Ok(settings.read().get_keybind_only_unread()),
            "only_starred" => Ok(settings.read().get_keybind_only_starred()),
            "scroll_up" => Ok(settings.read().get_keybind_article_view_up()),
            "scroll_down" => Ok(settings.read().get_keybind_article_view_down()),
            "scrap_content" => Ok(settings.read().get_keybind_article_view_scrap()),
            _ => {
                warn!("unexpected keybind id: {}", id);
                Err(SettingsErrorKind::InvalidKeybind.into())
            }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub all_articles: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub only_unread: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub only_starred: Option<String>,
}

impl KeybindingsGeneral {
    pub fn default() -> Self {
        KeybindingsGeneral {
            shortcut: Some("F1".to_owned()),
            refresh: Some("F5".to_owned()),
            search: Some("<ctl>F".to_owned()),
            quit: Some("<ctl>Q".to_owned()),
            all_articles: Some("<ctl>1".to_owned()),
            only_unread: Some("<ctl>2".to_owned()),
            only_starred: Some("<ctl>3".to_owned()),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub scrap_content: Option<String>,
}

impl KeybindingsArticleView {
    pub fn default() -> Self {
        KeybindingsArticleView {
            scroll_up: Some("I".into()),
            scroll_down: Some("U".into()),
            scrap_content: Some("C".into()),
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
    pub toggle_expanded: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub read: Option<String>,
}

impl KeybindingsFeedList {
    pub fn default() -> Self {
        KeybindingsFeedList {
            next: Some("<ctl>J".to_owned()),
            prev: Some("<ctl>K".to_owned()),
            toggle_expanded: Some("C".to_owned()),
            read: Some("<Shift>A".to_owned()),
        }
    }
}

//--------------------------------------------
// Shortcut window
//--------------------------------------------

pub struct NewsFlashShortcutWindow {
    pub widget: ShortcutsWindow,
}

impl NewsFlashShortcutWindow {
    pub fn new<D: IsA<Window> + GtkWindowExt>(settings_dialog: &D, settings: &Settings) -> Self {
        let ui_data = Resources::get("ui/shorcuts_window.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let mut ui_xml = str::from_utf8(ui_data.as_ref())
            .expect(GTK_RESOURCE_FILE_ERROR)
            .to_owned();

        ui_xml = Self::setup_shortcut(&ui_xml, "$SHORTCUT", settings.get_keybind_shortcut());
        ui_xml = Self::setup_shortcut(&ui_xml, "$REFRESH", settings.get_keybind_refresh());
        ui_xml = Self::setup_shortcut(&ui_xml, "$SEARCH", settings.get_keybind_search());
        ui_xml = Self::setup_shortcut(&ui_xml, "$QUIT", settings.get_keybind_quit());
        ui_xml = Self::setup_shortcut(&ui_xml, "$ALLARTICLES", settings.get_keybind_all_articles());
        ui_xml = Self::setup_shortcut(&ui_xml, "$ONLYUNREAD", settings.get_keybind_only_unread());
        ui_xml = Self::setup_shortcut(&ui_xml, "$ONLYSTARRED", settings.get_keybind_only_starred());
        ui_xml = Self::setup_shortcut(&ui_xml, "$NEXTART", settings.get_keybind_article_list_next());
        ui_xml = Self::setup_shortcut(&ui_xml, "$PREVART", settings.get_keybind_article_list_prev());
        ui_xml = Self::setup_shortcut(&ui_xml, "$TOGGLEREAD", settings.get_keybind_article_list_read());
        ui_xml = Self::setup_shortcut(&ui_xml, "$TOGGLEMARKED", settings.get_keybind_article_list_mark());
        ui_xml = Self::setup_shortcut(&ui_xml, "$OPENBROWSER", settings.get_keybind_article_list_open());
        ui_xml = Self::setup_shortcut(&ui_xml, "$SCROLLUP", settings.get_keybind_article_view_up());
        ui_xml = Self::setup_shortcut(&ui_xml, "$SCROLLDOWN", settings.get_keybind_article_view_down());
        ui_xml = Self::setup_shortcut(&ui_xml, "$SCRAPCONTENT", settings.get_keybind_article_view_scrap());
        ui_xml = Self::setup_shortcut(&ui_xml, "$NEXTFEED", settings.get_keybind_feed_list_next());
        ui_xml = Self::setup_shortcut(&ui_xml, "$PREVFEED", settings.get_keybind_feed_list_prev());
        ui_xml = Self::setup_shortcut(
            &ui_xml,
            "$TOGGLEEXPAND",
            settings.get_keybind_feed_list_toggle_expanded(),
        );
        ui_xml = Self::setup_shortcut(&ui_xml, "$ITEMREAD", settings.get_keybind_sidebar_set_read());

        let builder = BuilderHelper::new_from_xml(&ui_xml);
        let widget = builder.get::<ShortcutsWindow>("shortcuts-window");
        widget.set_transient_for(Some(settings_dialog));

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

        NewsFlashShortcutWindow { widget }
    }

    fn setup_shortcut(xml: &str, needle: &str, shortcut: Option<String>) -> String {
        match shortcut {
            Some(shortcut) => {
                let shortcut = shortcut.replace("&", "&amp;");
                let shortcut = shortcut.replace("<", "&lt;");
                let shortcut = shortcut.replace(">", "&gt;");
                xml.replacen(needle, &shortcut, 1)
            }
            None => xml.replacen(needle, "", 1),
        }
    }
}
