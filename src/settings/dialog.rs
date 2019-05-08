use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkHandle};
use super::theme_chooser::ThemeChooser;
use gtk::{Dialog, DialogExt, Window, GtkWindowExt, GtkWindowExtManual, Inhibit, FontButton, FontButtonExt, FontChooserExt,
    Label, LabelExt, ListBox, ListBoxExt, Stack, StackExt, Switch, SwitchExt, WidgetExt};
use glib::{object::IsA};
use gdk::{ModifierType, enums::key};
use gio::{ActionExt, ActionMapExt};
use news_flash::models::ArticleOrder;


pub struct SettingsDialog {
    widget: Dialog,
}

impl SettingsDialog {
    pub fn new<W: IsA<Window> + GtkWindowExt + ActionMapExt>(window: &W, settings: &GtkHandle<Settings>) -> Self {
        let builder = BuilderHelper::new("settings");

        let dialog = builder.get::<Dialog>("dialog");
        dialog.set_transient_for(window);

        Self::setup_ui_section(&builder, window, settings, &dialog);
        Self::setup_keybindings_section(&builder, settings, &dialog);

        SettingsDialog {
            widget: dialog,
        }
    }

    pub fn widget(&self) -> Dialog {
        self.widget.clone()
    }

    fn setup_ui_section<W: IsA<Window> + GtkWindowExt + ActionMapExt>(
        builder: &BuilderHelper,
        window: &W,
        settings: &GtkHandle<Settings>,
        dialog: &Dialog
    ) {
        let article_list_order_stack = builder.get::<Stack>("article_list_order_stack");
        Self::set_article_list_order_stack(settings, &article_list_order_stack);
        
        let settings_1 = settings.clone();
        let settings_2 = settings.clone();
        let settings_3 = settings.clone();
        let settings_4 = settings.clone();
        let settings_5 = settings.clone();

        let main_window = window.clone();
        let article_list_settings = builder.get::<ListBox>("article_list_settings");
        article_list_settings.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if "order" == row_name {
                    let order = settings_1.borrow().get_article_list_order();
                    settings_1.borrow_mut().set_article_list_order(order.invert()).unwrap();
                    Self::set_article_list_order_stack(&settings_1, &article_list_order_stack);
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
                }
            }
        });

        let dialog_window = dialog.clone();
        let main_window = window.clone();
        let article_view_settings = builder.get::<ListBox>("article_view_settings");
        let theme_label = builder.get::<Label>("theme_label");
        theme_label.set_label(settings.borrow().get_article_view_theme().name());
        article_view_settings.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if "theme" == row_name {
                    let main_window = main_window.clone();
                    let theme_label = theme_label.clone();
                    let settings = settings_2.clone();
                    let theme_chooser = ThemeChooser::new(&dialog_window, &settings_2);
                    theme_chooser.widget().connect_close(move |_dialog| {
                        theme_label.set_label(settings.borrow().get_article_view_theme().name());
                        if let Some(action) = main_window.lookup_action("redraw-article") {
                            action.activate(None);
                        }
                    });
                    theme_chooser.widget().present();
                }
            }
        });

        let main_window = window.clone();
        let allow_selection_switch = builder.get::<Switch>("allow_selection_switch");
        allow_selection_switch.set_state(settings.borrow().get_article_view_allow_select());
        allow_selection_switch.connect_state_set(move |_switch, is_set| {
            settings_3.borrow_mut().set_article_view_allow_select(is_set).unwrap();
            if let Some(action) = main_window.lookup_action("redraw-article") {
                action.activate(None);
            }
            Inhibit(false)
        });

        let main_window = window.clone();
        let font_label = builder.get::<Label>("font_label");
        let font_button = builder.get::<FontButton>("font_button");
        if let Some(font) = settings.borrow().get_article_view_font() {
            font_button.set_font(&font);
        }
        font_button.connect_font_set(move |button| {
            let font =  match button.get_font() {
                Some(font) => Some(font.to_string()),
                None => None,
            };
            settings_5.borrow_mut().set_article_view_font(font).unwrap();
            if let Some(action) = main_window.lookup_action("redraw-article") {
                action.activate(None);
            }
        });


        let main_window = window.clone();
        let use_system_font_switch = builder.get::<Switch>("use_system_font_switch");
        let have_custom_font = match settings.borrow().get_article_view_font() {
            Some(_) => true,
            None => false,
        };
        use_system_font_switch.set_state(!have_custom_font);
        font_button.set_sensitive(have_custom_font);
        font_label.set_sensitive(have_custom_font);
        use_system_font_switch.connect_state_set(move |_switch, is_set| {
            let font = if is_set {
                None
            } else {
                if let Some(font_name) = font_button.get_font() {
                    Some(font_name.to_string())
                } else {
                    None
                }
            };
            font_button.set_sensitive(!is_set);
            font_label.set_sensitive(!is_set);
            settings_4.borrow_mut().set_article_view_font(font).unwrap();
            if let Some(action) = main_window.lookup_action("redraw-article") {
                action.activate(None);
            }
            Inhibit(false)
        });
    }

    fn set_article_list_order_stack(settings: &GtkHandle<Settings>, article_list_order_stack: &Stack) {
        match settings.borrow().get_article_list_order() {
            ArticleOrder::NewestFirst => article_list_order_stack.set_visible_child_name("new"),
            ArticleOrder::OldestFirst => article_list_order_stack.set_visible_child_name("old"),
        }
    }

    fn setup_keybindings_section(
        builder: &BuilderHelper,
        settings: &GtkHandle<Settings>,
        _dialog: &Dialog
    ) {
        Self::setup_keybinding_row(builder, "next_article_label", settings.borrow().get_keybind_article_list_next());
        Self::setup_keybinding_row(builder, "previous_article_label", settings.borrow().get_keybind_article_list_prev());
        Self::setup_keybinding_row(builder, "toggle_read_label", settings.borrow().get_keybind_article_list_read());
        Self::setup_keybinding_row(builder, "toggle_marked_label", settings.borrow().get_keybind_article_list_mark());
        Self::setup_keybinding_row(builder, "open_browser_label", settings.borrow().get_keybind_article_list_open());
        
        Self::setup_keybinding_row(builder, "next_item_label", settings.borrow().get_keybind_feed_list_next());
        Self::setup_keybinding_row(builder, "previous_item_label", settings.borrow().get_keybind_feed_list_prev());
        Self::setup_keybinding_row(builder, "expand_category_label", settings.borrow().get_keybind_feed_list_expand());
        Self::setup_keybinding_row(builder, "collapse_category_label", settings.borrow().get_keybind_feed_list_collapse());
        Self::setup_keybinding_row(builder, "feed_read_label", settings.borrow().get_keybind_feed_list_read());
        
        Self::setup_keybinding_row(builder, "shortcuts_label", settings.borrow().get_keybind_shortcut());
        Self::setup_keybinding_row(builder, "refresh_label", settings.borrow().get_keybind_refresh());
        Self::setup_keybinding_row(builder, "search_label", settings.borrow().get_keybind_search());
        Self::setup_keybinding_row(builder, "quit_label", settings.borrow().get_keybind_quit());

        Self::setup_keybinding_row(builder, "scroll_up_label", settings.borrow().get_keybind_article_view_up());
        Self::setup_keybinding_row(builder, "scroll_down_label", settings.borrow().get_keybind_article_view_down());
    }

    fn setup_keybinding_row(
        builder: &BuilderHelper,
        label_id: &str,
        keybinding: Option<String>,
    ) {
        let label = builder.get::<Label>(label_id);
        let label_text = match keybinding {
            Some(keybinding) => {
                label.set_sensitive(true);
                let (keyval, modifier) = gtk::accelerator_parse(&keybinding);
                let keyval = Self::parse_keyval(keyval);
                let modifier = Self::parse_modifiers(modifier);
                match modifier {
                    Some(mut modifier) => {
                        modifier.push_str(&keyval);
                        modifier
                    },
                    None => keyval,
                }
            },
            None => {
                label.set_sensitive(false);
                "Disabled".to_owned()
            },
        };
        label.set_label(&label_text);
    }

    fn parse_keyval(keyval: u32) -> String {
        let keyval = gdk::keyval_to_upper(keyval);
        match gdk::keyval_to_unicode(keyval) {
            Some(keyval) => {
                let mut buffer : [u8; 4] = [0; 4];
                keyval.encode_utf8(&mut buffer).to_owned()
            },
            None => {
                match keyval {
                    key::Shift_L | key::Control_L | key::Alt_L | key::Meta_L | key::Super_L | key::Hyper_L |
                    key::Shift_R | key::Control_R | key::Alt_R | key::Meta_R | key::Super_R | key::Hyper_R => Self::get_modifier_label(keyval),
                    key::Left => "←".to_owned(),
                    key::Right => "→".to_owned(),
                    key::Down => "↓".to_owned(),
                    key::Up => "↑".to_owned(),
                    key::space => "␣".to_owned(),
                    key::Return => "⏎".to_owned(),
                    key::Page_Up => "⇑".to_owned(),
                    key::Page_Down => "⇓".to_owned(),
                    key::F1 => "F1".to_owned(),
                    key::F2 => "F2".to_owned(),
                    key::F3 => "F3".to_owned(),
                    key::F4 => "F4".to_owned(),
                    key::F5 => "F5".to_owned(),
                    key::F6 => "F6".to_owned(),
                    key::F7 => "F7".to_owned(),
                    key::F8 => "F8".to_owned(),
                    key::F9 => "F9".to_owned(),
                    key::F10 => "F10".to_owned(),
                    key::F11 => "F11".to_owned(),
                    key::F12 => "F12".to_owned(),
                    _ => "fixme".to_owned(),
                }
            },
        }
    }

    fn get_modifier_label(keyval: u32) -> String {
        let mut mod_string = String::new();
        match keyval {
            key::Shift_L | key::Control_L | key::Alt_L | key::Meta_L | key::Super_L | key::Hyper_L => { mod_string.push('L') },
            key::Shift_R | key::Control_R | key::Alt_R | key::Meta_R | key::Super_R | key::Hyper_R => { mod_string.push('R') },
            _ => {},
        }

        match keyval {
            key::Shift_L | key::Shift_R => { mod_string.push_str("Shift") },
            key::Control_L | key::Control_R => { mod_string.push_str("Ctrl") },
            key::Meta_L | key::Meta_R => { mod_string.push_str("Meta") },
            key::Super_L | key::Super_R => { mod_string.push_str("Super") },
            key::Hyper_L | key::Hyper_R => { mod_string.push_str("Hyper") },
            _ => {},
        }

        mod_string
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
            return None
        }

        Some(mod_string)
    }
}