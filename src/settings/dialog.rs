use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkHandle, GTK_BUILDER_ERROR};
use super::theme_chooser::ThemeChooser;
use super::keybindings::Keybindings;
use super::keybinding_editor::{KeybindingEditor, KeybindState};
use gtk::{Dialog, DialogExt, Window, GtkWindowExt, GtkWindowExtManual, Inhibit, FontButton, FontButtonExt, FontChooserExt,
    Label, LabelExt, ListBox, ListBoxExt, Stack, StackExt, Switch, SwitchExt, WidgetExt};
use glib::{object::IsA};
use gio::{ActionExt, ActionMapExt};
use news_flash::models::ArticleOrder;


pub struct SettingsDialog {
    widget: Dialog,
    settings: GtkHandle<Settings>,
    builder: BuilderHelper,
}

impl SettingsDialog {
    pub fn new<W: IsA<Window> + GtkWindowExt + ActionMapExt>(window: &W, settings: &GtkHandle<Settings>) -> Self {
        let builder = BuilderHelper::new("settings");

        let dialog = builder.get::<Dialog>("dialog");
        dialog.set_transient_for(window);

        let settings_dialog = SettingsDialog {
            widget: dialog,
            settings: settings.clone(),
            builder,
        };

        settings_dialog.setup_ui_section(window);
        settings_dialog.setup_keybindings_section();

        settings_dialog
    }

    pub fn widget(&self) -> Dialog {
        self.widget.clone()
    }

    fn setup_ui_section<W: IsA<Window> + GtkWindowExt + ActionMapExt>(&self, window: &W) {
        let article_list_order_stack = self.builder.get::<Stack>("article_list_order_stack");
        Self::set_article_list_order_stack(&self.settings, &article_list_order_stack);
        
        let settings_1 = self.settings.clone();
        let settings_2 = self.settings.clone();
        let settings_3 = self.settings.clone();
        let settings_4 = self.settings.clone();
        let settings_5 = self.settings.clone();

        let main_window = window.clone();
        let article_list_settings = self.builder.get::<ListBox>("article_list_settings");
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

        let dialog_window = self.widget.clone();
        let main_window = window.clone();
        let article_view_settings = self.builder.get::<ListBox>("article_view_settings");
        let theme_label = self.builder.get::<Label>("theme_label");
        theme_label.set_label(self.settings.borrow().get_article_view_theme().name());
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
        let allow_selection_switch = self.builder.get::<Switch>("allow_selection_switch");
        allow_selection_switch.set_state(self.settings.borrow().get_article_view_allow_select());
        allow_selection_switch.connect_state_set(move |_switch, is_set| {
            settings_3.borrow_mut().set_article_view_allow_select(is_set).unwrap();
            if let Some(action) = main_window.lookup_action("redraw-article") {
                action.activate(None);
            }
            Inhibit(false)
        });

        let main_window = window.clone();
        let font_label = self.builder.get::<Label>("font_label");
        let font_button = self.builder.get::<FontButton>("font_button");
        if let Some(font) = self.settings.borrow().get_article_view_font() {
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
        let use_system_font_switch = self.builder.get::<Switch>("use_system_font_switch");
        let have_custom_font = match self.settings.borrow().get_article_view_font() {
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

    fn setup_keybindings_section(&self) {
        let article_keys_list = self.builder.get::<ListBox>("article_keys_list");
        self.setup_keybinding_row(&article_keys_list, "next_article", self.settings.borrow().get_keybind_article_list_next());
        self.setup_keybinding_row(&article_keys_list, "previous_article", self.settings.borrow().get_keybind_article_list_prev());
        self.setup_keybinding_row(&article_keys_list, "toggle_read", self.settings.borrow().get_keybind_article_list_read());
        self.setup_keybinding_row(&article_keys_list, "toggle_marked", self.settings.borrow().get_keybind_article_list_mark());
        self.setup_keybinding_row(&article_keys_list, "open_browser", self.settings.borrow().get_keybind_article_list_open());
        
        let feed_keys_list = self.builder.get::<ListBox>("feed_keys_list");
        self.setup_keybinding_row(&feed_keys_list, "next_item", self.settings.borrow().get_keybind_feed_list_next());
        self.setup_keybinding_row(&feed_keys_list, "previous_item", self.settings.borrow().get_keybind_feed_list_prev());
        self.setup_keybinding_row(&feed_keys_list, "expand_category", self.settings.borrow().get_keybind_feed_list_expand());
        self.setup_keybinding_row(&feed_keys_list, "collapse_category", self.settings.borrow().get_keybind_feed_list_collapse());
        self.setup_keybinding_row(&feed_keys_list, "feed_read", self.settings.borrow().get_keybind_feed_list_read());
        
        let general_keys_list = self.builder.get::<ListBox>("general_keys_list");
        self.setup_keybinding_row(&general_keys_list, "shortcuts", self.settings.borrow().get_keybind_shortcut());
        self.setup_keybinding_row(&general_keys_list, "refresh", self.settings.borrow().get_keybind_refresh());
        self.setup_keybinding_row(&general_keys_list, "search", self.settings.borrow().get_keybind_search());
        self.setup_keybinding_row(&general_keys_list, "quit", self.settings.borrow().get_keybind_quit());

        let article_view_keys_list = self.builder.get::<ListBox>("article_view_keys_list");
        self.setup_keybinding_row(&article_view_keys_list, "scroll_up", self.settings.borrow().get_keybind_article_view_up());
        self.setup_keybinding_row(&article_view_keys_list, "scroll_down", self.settings.borrow().get_keybind_article_view_down());
    }

    fn setup_keybinding_row(
        &self,
        list: &ListBox,
        id: &str,
        keybinding: Option<String>,
    ) {
        let label = self.builder.get::<Label>(&format!("{}_label", id));
        Self::keybind_label_text(keybinding, &label);

        let dialog = self.widget.clone();
        let id = id.to_owned();
        let info_label = self.builder.get::<Label>(&format!("{}_info_label", id));
        let info_text = info_label.get_label().expect(GTK_BUILDER_ERROR);
        let settings = self.settings.clone();
        list.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if id == row_name {
                    let id = id.clone();
                    let label = label.clone();
                    let settings = settings.clone();
                    let editor = KeybindingEditor::new(&dialog, &info_text);
                    editor.widget().present();
                    editor.widget().connect_close(move |_dialog| {
                        let _settings = settings.clone();
                        match &*editor.keybinding.borrow() {
                            KeybindState::Canceled | KeybindState::Illegal => {},
                            KeybindState::Disabled => { 
                                Keybindings::write_keybinding(&id, None, &settings).unwrap();
                                Self::keybind_label_text(None, &label);
                            },
                            KeybindState::Enabled(keybind) => {
                                Keybindings::write_keybinding(&id, Some(keybind.clone()), &settings).unwrap();
                                Self::keybind_label_text(Some(keybind.clone()), &label);
                            },
                        }
                    });
                }
            }
        });
    }

    fn keybind_label_text(keybinding: Option<String>, label: &Label) {
        let label_text = match keybinding {
            Some(keybinding) => {
                label.set_sensitive(true);
                Keybindings::parse_shortcut_string(&keybinding)
                    .expect("Failed parsing saved shortcut. This should never happen!")
            },
            None => {
                label.set_sensitive(false);
                "Disabled".to_owned()
            },
        };
        label.set_label(&label_text);
    }
}