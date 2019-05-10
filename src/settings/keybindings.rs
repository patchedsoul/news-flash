use serde_derive::{Deserialize, Serialize};
use gtk::{Box, Cast, ContainerExt, Window, GtkWindowExt, ShortcutsWindow, Stack, StackExt, BinExt, WidgetExt};
use glib::{object::IsA};
use gdk::{ModifierType, enums::key};
use crate::util::{BuilderHelper, GtkHandle, GTK_RESOURCE_FILE_ERROR};
use crate::settings::Settings;
use crate::Resources;
use std::str;
use log::warn;
use failure::{Error, format_err};

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

    pub fn parse_shortcut_string(keybinding: &str) -> Option<String> {
        let (keyval, modifier) = gtk::accelerator_parse(&keybinding);
        Self::parse_shortcut(keyval, &modifier)
    }

    pub fn parse_shortcut(keyval: u32, modifier: &ModifierType) -> Option<String> {
        let keyval = Self::parse_keyval(keyval);
        let modifier = Self::parse_modifiers(modifier);
        match keyval {
            None => None,
            Some(keyval) => {
                match modifier {
                    Some(mut modifier) => {
                        modifier.push_str(&keyval);
                        Some(modifier)
                    },
                    None => Some(keyval),
                }
            },
        }
    }

    fn parse_keyval(keyval: u32) -> Option<String> {
        let keyval = gdk::keyval_to_upper(keyval);

        let manual_parsed = match keyval {
            key::Shift_L | key::Control_L | key::Alt_L | key::Meta_L | key::Super_L | key::Hyper_L |
            key::Shift_R | key::Control_R | key::Alt_R | key::Meta_R | key::Super_R | key::Hyper_R => Self::get_modifier_label(keyval),
            key::Left => Some("←".to_owned()),
            key::Right => Some("→".to_owned()),
            key::Down => Some("↓".to_owned()),
            key::Up => Some("↑".to_owned()),
            key::space => Some("␣".to_owned()),
            key::Return => Some("⏎".to_owned()),
            key::Delete => Some("Del".to_owned()),
            key::Page_Up => Some("⇑".to_owned()),
            key::Page_Down => Some("⇓".to_owned()),
            key::BackSpace => Some("Backspace".to_owned()),
            key::F1 => Some("F1".to_owned()),
            key::F2 => Some("F2".to_owned()),
            key::F3 => Some("F3".to_owned()),
            key::F4 => Some("F4".to_owned()),
            key::F5 => Some("F5".to_owned()),
            key::F6 => Some("F6".to_owned()),
            key::F7 => Some("F7".to_owned()),
            key::F8 => Some("F8".to_owned()),
            key::F9 => Some("F9".to_owned()),
            key::F10 => Some("F10".to_owned()),
            key::F11 => Some("F11".to_owned()),
            key::F12 => Some("F12".to_owned()),
            _ => None,
        };

        if manual_parsed.is_some() {
            return manual_parsed;
        }

        match gdk::keyval_to_unicode(keyval) {
            Some(keyval) => {
                let mut buffer : [u8; 4] = [0; 4];
                Some(keyval.encode_utf8(&mut buffer).to_owned())
            },
            None => None,
        }
    }

    fn get_modifier_label(keyval: u32) -> Option<String> {
        let mut mod_string = String::new();
        match keyval {
            key::Shift_L | key::Control_L | key::Alt_L | key::Meta_L | key::Super_L | key::Hyper_L => { mod_string.push('L') },
            key::Shift_R | key::Control_R | key::Alt_R | key::Meta_R | key::Super_R | key::Hyper_R => { mod_string.push('R') },
            _ => return None,
        }

        match keyval {
            key::Shift_L | key::Shift_R => { mod_string.push_str("Shift") },
            key::Control_L | key::Control_R => { mod_string.push_str("Ctrl") },
            key::Meta_L | key::Meta_R => { mod_string.push_str("Meta") },
            key::Super_L | key::Super_R => { mod_string.push_str("Super") },
            key::Hyper_L | key::Hyper_R => { mod_string.push_str("Hyper") },
            _ => return None,
        }

        Some(mod_string)
    }

    fn parse_modifiers(modifier: &ModifierType) -> Option<String> {
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
            return None
        }

        Some(mod_string)
    }

    pub fn clean_modifier(modifier: &ModifierType) -> ModifierType {
        let mut modifier = modifier.clone();

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

    pub fn write_keybinding(id: &str, keybinding: Option<String>, settings: &GtkHandle<Settings>) -> Result<(), Error> {
        match id {
            "next_article" => settings.borrow_mut().set_keybind_article_list_next(keybinding),
            "previous_article" => settings.borrow_mut().set_keybind_article_list_prev(keybinding),
            "toggle_read" => settings.borrow_mut().set_keybind_article_list_read(keybinding),
            "toggle_marked" => settings.borrow_mut().set_keybind_article_list_mark(keybinding),
            "open_browser" => settings.borrow_mut().set_keybind_article_list_open(keybinding),
            "feed_keys_list" => settings.borrow_mut().set_keybind_feed_list_next(keybinding),
            "previous_item" => settings.borrow_mut().set_keybind_feed_list_prev(keybinding),
            "expand_category" => settings.borrow_mut().set_keybind_feed_list_expand(keybinding),
            "collapse_category" => settings.borrow_mut().set_keybind_feed_list_collapse(keybinding),
            "feed_read" => settings.borrow_mut().set_keybind_feed_list_read(keybinding),
            "shortcuts" => settings.borrow_mut().set_keybind_shortcut(keybinding),
            "refresh" => settings.borrow_mut().set_keybind_refresh(keybinding),
            "search" => settings.borrow_mut().set_keybind_search(keybinding),
            "quit" => settings.borrow_mut().set_keybind_quit(keybinding),
            "scroll_up" => settings.borrow_mut().set_keybind_article_view_up(keybinding),
            "scroll_down" => settings.borrow_mut().set_keybind_article_view_down(keybinding),
            _ => {
                warn!("unexpected keybind id: {}", id);
                Err(format_err!("some err"))
            },
        }
    }

    pub fn read_keybinding(id: &str, settings: &GtkHandle<Settings>) -> Result<Option<String>, Error> {
        match id {
            "next_article" => Ok(settings.borrow_mut().get_keybind_article_list_next()),
            "previous_article" => Ok(settings.borrow_mut().get_keybind_article_list_prev()),
            "toggle_read" => Ok(settings.borrow_mut().get_keybind_article_list_read()),
            "toggle_marked" => Ok(settings.borrow_mut().get_keybind_article_list_mark()),
            "open_browser" => Ok(settings.borrow_mut().get_keybind_article_list_open()),
            "feed_keys_list" => Ok(settings.borrow_mut().get_keybind_feed_list_next()),
            "previous_item" => Ok(settings.borrow_mut().get_keybind_feed_list_prev()),
            "expand_category" => Ok(settings.borrow_mut().get_keybind_feed_list_expand()),
            "collapse_category" => Ok(settings.borrow_mut().get_keybind_feed_list_collapse()),
            "feed_read" => Ok(settings.borrow_mut().get_keybind_feed_list_read()),
            "shortcuts" => Ok(settings.borrow_mut().get_keybind_shortcut()),
            "refresh" => Ok(settings.borrow_mut().get_keybind_refresh()),
            "search" => Ok(settings.borrow_mut().get_keybind_search()),
            "quit" => Ok(settings.borrow_mut().get_keybind_quit()),
            "scroll_up" => Ok(settings.borrow_mut().get_keybind_article_view_up()),
            "scroll_down" => Ok(settings.borrow_mut().get_keybind_article_view_down()),
            _ => {
                warn!("unexpected keybind id: {}", id);
                Err(format_err!("some err"))
            },
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