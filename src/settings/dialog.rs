use super::general::SyncInterval;
use super::keybinding_editor::{KeybindState, KeybindingEditor};
use super::keybindings::Keybindings;
use super::theme_chooser::ThemeChooser;
use crate::app::Action;
use crate::settings::Settings;
use crate::util::{BuilderHelper, Util, GTK_BUILDER_ERROR};
use gdk::{EventMask, EventType};
use glib::{object::Cast, Sender};
use gtk::{
    DialogExt, EventBox, FontButton, FontButtonExt, FontChooserExt, GtkWindowExt, GtkWindowExtManual, Inhibit, Label,
    LabelExt, ListBox, ListBoxExt, ListBoxRowExt, Popover, PopoverExt, Settings as GtkSettings,
    SettingsExt as GtkSettingsExt, Switch, SwitchExt, WidgetExt, WidgetExtManual, Window,
};
use libhandy::{ActionRow, PreferencesRowExt};
use news_flash::models::ArticleOrder;
use parking_lot::RwLock;
use std::rc::Rc;

pub struct SettingsDialog {
    pub widget: Window,
    settings: Rc<RwLock<Settings>>,
    builder: BuilderHelper,
}

impl SettingsDialog {
    pub fn new(window: &gtk::ApplicationWindow, sender: &Sender<Action>, settings: &Rc<RwLock<Settings>>) -> Self {
        let builder = BuilderHelper::new("settings");

        let dialog = builder.get::<Window>("dialog");
        dialog.set_transient_for(Some(window));

        let settings_dialog = SettingsDialog {
            widget: dialog,
            settings: settings.clone(),
            builder,
        };

        settings_dialog.setup_ui_section(sender);
        settings_dialog.setup_keybindings_section(sender);

        settings_dialog
    }

    fn setup_ui_section(&self, sender: &Sender<Action>) {
        let settings_1 = self.settings.clone();
        let settings_2 = self.settings.clone();
        let settings_3 = self.settings.clone();
        let settings_4 = self.settings.clone();
        let settings_5 = self.settings.clone();
        let settings_6 = self.settings.clone();
        let settings_7 = self.settings.clone();
        let settings_8 = self.settings.clone();
        let settings_9 = self.settings.clone();

        let sender_1 = sender.clone();
        let sender_2 = sender.clone();
        let sender_3 = sender.clone();
        let sender_4 = sender.clone();
        let sender_5 = sender.clone();
        let sender_6 = sender.clone();
        let sender_7 = sender.clone();
        let sender_8 = sender.clone();
        let sender_9 = sender.clone();

        let keep_running_switch = self.builder.get::<Switch>("keep_running_switch");
        keep_running_switch.set_state(self.settings.read().get_keep_running_in_background());
        keep_running_switch.connect_state_set(move |_switch, is_set| {
            if settings_1.write().set_keep_running_in_background(is_set).is_err() {
                Util::send(
                    &sender_1,
                    Action::ErrorSimpleMessage("Failed to set setting 'keep running'.".to_owned()),
                );
            }
            Inhibit(false)
        });

        let dark_theme_switch = self.builder.get::<Switch>("dark_theme_switch");
        dark_theme_switch.set_state(self.settings.read().get_prefer_dark_theme());
        dark_theme_switch.connect_state_set(move |_switch, is_set| {
            if settings_2.write().set_prefer_dark_theme(is_set).is_ok() {
                if let Some(settings) = GtkSettings::get_default() {
                    settings.set_property_gtk_application_prefer_dark_theme(is_set);
                }
            } else {
                Util::send(
                    &sender_2,
                    Action::ErrorSimpleMessage("Failed to set setting 'dark theme'.".to_owned()),
                );
            }
            Inhibit(false)
        });

        let sync_label = self.builder.get::<Label>("sync_label");
        let sync_pop = self.builder.get::<Popover>("sync_pop");
        let sync_list = self.builder.get::<ListBox>("sync_list");
        sync_list.connect_row_activated(move |_list, row| {
            sync_pop.popdown();
            let sync_interval = match row.get_index() {
                0 => SyncInterval::Never,
                2 => SyncInterval::QuaterHour,
                4 => SyncInterval::HalfHour,
                6 => SyncInterval::Hour,
                8 => SyncInterval::TwoHour,

                _ => SyncInterval::Never,
            };
            sync_label.set_label(&sync_interval.to_string());
            if settings_3.write().set_sync_interval(sync_interval).is_ok() {
                Util::send(&sender_3, Action::ScheduleSync);
            } else {
                Util::send(
                    &sender_3,
                    Action::ErrorSimpleMessage("Failed to set setting 'sync interval'.".to_owned()),
                );
            }
        });

        let sync_label = self.builder.get::<Label>("sync_label");
        sync_label.set_label(&self.settings.read().get_sync_interval().to_string());
        let sync_event = self.builder.get::<EventBox>("sync_event");
        sync_event.set_events(EventMask::BUTTON_PRESS_MASK);
        let sync_pop = self.builder.get::<Popover>("sync_pop");
        sync_event.connect_button_press_event(move |_eventbox, event| {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false);
                }
                _ => {}
            }

            sync_pop.popup();
            Inhibit(false)
        });

        let sync_pop = self.builder.get::<Popover>("sync_pop");
        let sync_row = self.builder.get::<ActionRow>("sync_row");
        if let Some(listbox) = sync_row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                listbox.connect_row_activated(move |_list, row| {
                    if let Some(name) = row.get_name() {
                        if name == "sync_row" {
                            sync_pop.popup();
                        }
                    }
                });
            }
        }

        let article_order_pop = self.builder.get::<Popover>("article_order_pop");
        let article_order_label = self.builder.get::<Label>("article_order_label");
        let article_order_list = self.builder.get::<ListBox>("article_order_list");
        article_order_list.connect_row_activated(move |_list, row| {
            article_order_pop.popdown();
            let new_order = match row.get_index() {
                0 => ArticleOrder::NewestFirst,
                _ => ArticleOrder::OldestFirst,
            };
            article_order_label.set_label(new_order.to_str());
            if settings_4.write().set_article_list_order(new_order).is_ok() {
                Util::send(&sender_4, Action::UpdateArticleList);
            } else {
                Util::send(
                    &sender_4,
                    Action::ErrorSimpleMessage("Failed to set setting 'article order'.".to_owned()),
                );
            }
        });

        let article_order_label = self.builder.get::<Label>("article_order_label");
        article_order_label.set_label(self.settings.read().get_article_list_order().to_str());
        let article_order_event = self.builder.get::<EventBox>("article_order_event");
        article_order_event.set_events(EventMask::BUTTON_PRESS_MASK);
        let article_order_pop = self.builder.get::<Popover>("article_order_pop");
        article_order_event.connect_button_press_event(move |_eventbox, event| {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false);
                }
                _ => {}
            }

            article_order_pop.popup();
            Inhibit(false)
        });

        let article_order_pop = self.builder.get::<Popover>("article_order_pop");
        let article_order_row = self.builder.get::<ActionRow>("article_order_row");
        if let Some(listbox) = article_order_row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                listbox.connect_row_activated(move |_list, row| {
                    if let Some(name) = row.get_name() {
                        if name == "article_order_row" {
                            article_order_pop.popup();
                        }
                    }
                });
            }
        }

        let article_theme_label = self.builder.get::<Label>("article_theme_label");
        let article_theme_event = self.builder.get::<EventBox>("article_theme_event");
        article_theme_event.set_events(EventMask::BUTTON_PRESS_MASK);
        article_theme_event.connect_button_press_event(move |eventbox, event| {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false);
                }
                _ => {}
            }

            let settings = settings_5.clone();
            let article_theme_label = article_theme_label.clone();
            let theme_chooser = ThemeChooser::new(eventbox, &sender_5, &settings);
            let sender = sender_5.clone();
            theme_chooser.widget().connect_closed(move |_pop| {
                article_theme_label.set_label(settings.read().get_article_view_theme().name());
                Util::send(&sender, Action::RedrawArticle);
            });
            theme_chooser.widget().popup();

            Inhibit(false)
        });

        let article_theme_label = self.builder.get::<Label>("article_theme_label");
        let article_theme_event = self.builder.get::<EventBox>("article_theme_event");
        let article_theme_row = self.builder.get::<ActionRow>("article_theme_row");
        if let Some(listbox) = article_theme_row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                listbox.connect_row_activated(move |_list, row| {
                    if let Some(name) = row.get_name() {
                        if name == "article_theme_row" {
                            let settings = settings_6.clone();
                            let article_theme_label = article_theme_label.clone();
                            let sender = sender_6.clone();
                            let theme_chooser = ThemeChooser::new(&article_theme_event, &sender_6, &settings);
                            theme_chooser.widget().connect_closed(move |_pop| {
                                article_theme_label.set_label(settings.read().get_article_view_theme().name());
                                Util::send(&sender, Action::RedrawArticle);
                            });
                            theme_chooser.widget().popup();
                        }
                    }
                });
            }
        }

        let allow_selection_switch = self.builder.get::<Switch>("allow_selection_switch");
        allow_selection_switch.set_state(self.settings.read().get_article_view_allow_select());
        allow_selection_switch.connect_state_set(move |_switch, is_set| {
            if settings_7.write().set_article_view_allow_select(is_set).is_ok() {
                Util::send(&sender_7, Action::RedrawArticle);
            } else {
                Util::send(
                    &sender_7,
                    Action::ErrorSimpleMessage("Failed to set setting 'allow article selection'.".to_owned()),
                );
            }
            Inhibit(false)
        });

        let font_row = self.builder.get::<ActionRow>("font_row");
        let font_button = self.builder.get::<FontButton>("font_button");
        if let Some(font) = self.settings.read().get_article_view_font() {
            font_button.set_font(&font);
        }
        font_button.connect_font_set(move |button| {
            let font = match button.get_font() {
                Some(font) => Some(font.to_string()),
                None => None,
            };
            if settings_8.write().set_article_view_font(font).is_ok() {
                Util::send(&sender_8, Action::RedrawArticle);
            } else {
                Util::send(
                    &sender_8,
                    Action::ErrorSimpleMessage("Failed to set setting 'article font'.".to_owned()),
                );
            }
        });

        let use_system_font_switch = self.builder.get::<Switch>("use_system_font_switch");
        let have_custom_font = self.settings.read().get_article_view_font().is_some();

        use_system_font_switch.set_state(!have_custom_font);
        font_button.set_sensitive(have_custom_font);
        font_row.set_sensitive(have_custom_font);
        use_system_font_switch.connect_state_set(move |_switch, is_set| {
            let font = if is_set {
                None
            } else if let Some(font_name) = font_button.get_font() {
                Some(font_name.to_string())
            } else {
                None
            };
            font_button.set_sensitive(!is_set);
            font_row.set_sensitive(!is_set);
            if settings_9.write().set_article_view_font(font).is_ok() {
                Util::send(&sender_9, Action::RedrawArticle);
            } else {
                Util::send(
                    &sender_9,
                    Action::ErrorSimpleMessage("Failed to set setting 'use system font'.".to_owned()),
                );
            }
            Inhibit(false)
        });
    }

    fn setup_keybindings_section(&self, sender: &Sender<Action>) {
        self.setup_keybinding_row(
            "next_article",
            self.settings.read().get_keybind_article_list_next(),
            sender,
        );
        self.setup_keybinding_row(
            "previous_article",
            self.settings.read().get_keybind_article_list_prev(),
            sender,
        );
        self.setup_keybinding_row(
            "toggle_read",
            self.settings.read().get_keybind_article_list_read(),
            sender,
        );
        self.setup_keybinding_row(
            "toggle_marked",
            self.settings.read().get_keybind_article_list_mark(),
            sender,
        );
        self.setup_keybinding_row(
            "open_browser",
            self.settings.read().get_keybind_article_list_open(),
            sender,
        );

        self.setup_keybinding_row("next_item", self.settings.read().get_keybind_feed_list_next(), sender);
        self.setup_keybinding_row(
            "previous_item",
            self.settings.read().get_keybind_feed_list_prev(),
            sender,
        );
        self.setup_keybinding_row(
            "toggle_category_expanded",
            self.settings.read().get_keybind_feed_list_toggle_expanded(),
            sender,
        );
        self.setup_keybinding_row(
            "sidebar_set_read",
            self.settings.read().get_keybind_sidebar_set_read(),
            sender,
        );

        self.setup_keybinding_row("shortcuts", self.settings.read().get_keybind_shortcut(), sender);
        self.setup_keybinding_row("refresh", self.settings.read().get_keybind_refresh(), sender);
        self.setup_keybinding_row("search", self.settings.read().get_keybind_search(), sender);
        self.setup_keybinding_row("quit", self.settings.read().get_keybind_quit(), sender);

        self.setup_keybinding_row("all_articles", self.settings.read().get_keybind_all_articles(), sender);
        self.setup_keybinding_row("only_unread", self.settings.read().get_keybind_only_unread(), sender);
        self.setup_keybinding_row("only_starred", self.settings.read().get_keybind_only_starred(), sender);

        self.setup_keybinding_row("scroll_up", self.settings.read().get_keybind_article_view_up(), sender);
        self.setup_keybinding_row(
            "scroll_down",
            self.settings.read().get_keybind_article_view_down(),
            sender,
        );
    }

    fn setup_keybinding_row(&self, id: &str, keybinding: Option<String>, sender: &Sender<Action>) {
        let label = self.builder.get::<Label>(&format!("{}_label", id));
        Self::keybind_label_text(keybinding.clone(), &label);
        let row_name = format!("{}_row", id);
        let row = self.builder.get::<ActionRow>(&row_name);
        if let Some(listbox) = row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                let dialog = self.widget.clone();
                let id = id.to_owned();
                let info_text = row.get_title().expect(GTK_BUILDER_ERROR);
                let settings = self.settings.clone();
                let sender = sender.clone();
                listbox.connect_row_activated(move |_list, row| {
                    if let Some(name) = row.get_name() {
                        if name.as_str() == row_name {
                            let id = id.clone();
                            let label = label.clone();
                            let settings = settings.clone();
                            let sender = sender.clone();
                            let editor = KeybindingEditor::new(&dialog, &info_text);
                            editor.widget().present();
                            editor.widget().connect_close(move |_dialog| {
                                let _settings = settings.clone();
                                match &*editor.keybinding.borrow() {
                                    KeybindState::Canceled | KeybindState::Illegal => {}
                                    KeybindState::Disabled => {
                                        if Keybindings::write_keybinding(&id, None, &settings).is_ok() {
                                            Self::keybind_label_text(None, &label);
                                        } else {
                                            Util::send(
                                                &sender,
                                                Action::ErrorSimpleMessage("Failed to write keybinding.".to_owned()),
                                            );
                                        }
                                    }
                                    KeybindState::Enabled(keybind) => {
                                        if Keybindings::write_keybinding(&id, Some(keybind.clone()), &settings).is_ok()
                                        {
                                            Self::keybind_label_text(Some(keybind.clone()), &label);
                                        } else {
                                            Util::send(
                                                &sender,
                                                Action::ErrorSimpleMessage("Failed to write keybinding.".to_owned()),
                                            );
                                        }
                                    }
                                }
                            });
                        }
                    }
                });
            }
        }
    }

    fn keybind_label_text(keybinding: Option<String>, label: &Label) {
        let label_text = match keybinding {
            Some(keybinding) => {
                label.set_sensitive(true);
                Keybindings::parse_shortcut_string(&keybinding)
                    .expect("Failed parsing saved shortcut. This should never happen!")
            }
            None => {
                label.set_sensitive(false);
                "Disabled".to_owned()
            }
        };
        label.set_label(&label_text);
    }
}
