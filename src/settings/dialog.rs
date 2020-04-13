use super::general::SyncInterval;
use super::keybinding_editor::{KeybindState, KeybindingEditor};
use super::keybindings::Keybindings;
use super::theme_chooser::ThemeChooser;
use crate::app::Action;
use crate::settings::Settings;
use crate::util::{BuilderHelper, Util, GTK_BUILDER_ERROR};
use gdk::{EventMask, EventType};
use glib::{clone, object::Cast, Sender};
use gtk::{
    prelude::GtkWindowExtManual, prelude::WidgetExtManual, DialogExt, EventBox, FontButton, FontButtonExt,
    FontChooserExt, GtkWindowExt, Inhibit, Label, LabelExt, ListBox, ListBoxExt, ListBoxRowExt, Popover, PopoverExt,
    Settings as GtkSettings, SettingsExt as GtkSettingsExt, Switch, SwitchExt, WidgetExt, Window,
};
use libhandy::{ActionRow, PreferencesRowExt};
use news_flash::models::ArticleOrder;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SettingsDialog {
    pub widget: Window,
    keep_running_switch: Switch,
    dark_theme_switch: Switch,
    sync_label: Label,
    sync_pop: Popover,
    sync_list: ListBox,
    sync_event: EventBox,
    sync_row: ActionRow,
    article_order_pop: Popover,
    article_order_list: ListBox,
    article_order_label: Label,
    article_order_event: EventBox,
    article_order_row: ActionRow,
    article_theme_label: Label,
    article_theme_row: ActionRow,
    article_theme_event: EventBox,
    allow_selection_switch: Switch,
    font_row: ActionRow,
    font_button: FontButton,
    use_system_font_switch: Switch,
    settings: Arc<RwLock<Settings>>,
    builder: BuilderHelper,
}

impl SettingsDialog {
    pub fn new(window: &gtk::ApplicationWindow, sender: &Sender<Action>, settings: &Arc<RwLock<Settings>>) -> Self {
        let have_custom_font = settings.read().get_article_view_font().is_some();

        let builder = BuilderHelper::new("settings");

        let dialog = builder.get::<Window>("dialog");
        dialog.set_transient_for(Some(window));

        let keep_running_switch = builder.get::<Switch>("keep_running_switch");
        keep_running_switch.set_state(settings.read().get_keep_running_in_background());

        let dark_theme_switch = builder.get::<Switch>("dark_theme_switch");
        dark_theme_switch.set_state(settings.read().get_prefer_dark_theme());

        let sync_label = builder.get::<Label>("sync_label");
        sync_label.set_label(&settings.read().get_sync_interval().to_string());

        let sync_pop = builder.get::<Popover>("sync_pop");
        let sync_list = builder.get::<ListBox>("sync_list");

        let sync_event = builder.get::<EventBox>("sync_event");
        sync_event.set_events(EventMask::BUTTON_PRESS_MASK);

        let sync_row = builder.get::<ActionRow>("sync_row");
        let article_order_pop = builder.get::<Popover>("article_order_pop");

        let article_order_label = builder.get::<Label>("article_order_label");
        article_order_label.set_label(settings.read().get_article_list_order().to_str());

        let article_order_list = builder.get::<ListBox>("article_order_list");
        let article_order_row = builder.get::<ActionRow>("article_order_row");

        let article_order_event = builder.get::<EventBox>("article_order_event");
        article_order_event.set_events(EventMask::BUTTON_PRESS_MASK);

        let article_theme_label = builder.get::<Label>("article_theme_label");
        let article_theme_row = builder.get::<ActionRow>("article_theme_row");
        let article_theme_event = builder.get::<EventBox>("article_theme_event");
        article_theme_event.set_events(EventMask::BUTTON_PRESS_MASK);

        let allow_selection_switch = builder.get::<Switch>("allow_selection_switch");
        allow_selection_switch.set_state(settings.read().get_article_view_allow_select());

        let font_row = builder.get::<ActionRow>("font_row");
        font_row.set_sensitive(have_custom_font);

        let font_button = builder.get::<FontButton>("font_button");
        font_button.set_sensitive(have_custom_font);
        if let Some(font) = settings.read().get_article_view_font() {
            font_button.set_font(&font);
        }

        let use_system_font_switch = builder.get::<Switch>("use_system_font_switch");
        use_system_font_switch.set_state(!have_custom_font);

        let settings_dialog = SettingsDialog {
            widget: dialog,
            keep_running_switch,
            dark_theme_switch,
            sync_label,
            sync_pop,
            sync_list,
            sync_event,
            sync_row,
            article_order_pop,
            article_order_label,
            article_order_list,
            article_order_event,
            article_order_row,
            article_theme_label,
            article_theme_row,
            article_theme_event,
            allow_selection_switch,
            font_row,
            font_button,
            use_system_font_switch,
            settings: settings.clone(),
            builder,
        };

        settings_dialog.setup_ui_section(sender);
        settings_dialog.setup_keybindings_section(sender);

        settings_dialog
    }

    fn setup_ui_section(&self, sender: &Sender<Action>) {
        self.keep_running_switch.connect_state_set(clone!(
            @weak self.settings as settings,
            @strong sender => @default-panic, move |_switch, is_set|
        {
            if settings.write().set_keep_running_in_background(is_set).is_err() {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to set setting 'keep running'.".to_owned()),
                );
            }
            Inhibit(false)
        }));

        self.dark_theme_switch.connect_state_set(clone!(
            @weak self.settings as settings,
            @strong sender => @default-panic, move |_switch, is_set| {
            if settings.write().set_prefer_dark_theme(is_set).is_ok() {
                if let Some(settings) = GtkSettings::get_default() {
                    settings.set_property_gtk_application_prefer_dark_theme(is_set);
                    Util::send(&sender, Action::RedrawArticle);
                }
            } else {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to set setting 'dark theme'.".to_owned()),
                );
            }
            Inhibit(false)
        }));

        self.sync_list.connect_row_activated(clone!(
            @weak self.settings as settings,
            @weak self.sync_pop as sync_pop,
            @weak self.sync_label as sync_label,
            @strong sender => move |_list, row| {
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
            if settings.write().set_sync_interval(sync_interval).is_ok() {
                Util::send(&sender, Action::ScheduleSync);
            } else {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to set setting 'sync interval'.".to_owned()),
                );
            }
        }));

        self.sync_event.connect_button_press_event(clone!(
            @weak self.sync_pop as sync_pop => @default-panic, move |_eventbox, event|
        {
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
        }));

        if let Some(listbox) = self.sync_row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                listbox.connect_row_activated(clone!(@weak self.sync_pop as sync_pop => move |_list, row| {
                    if let Some(name) = row.get_widget_name() {
                        if name == "sync_row" {
                            sync_pop.popup();
                        }
                    }
                }));
            }
        }

        self.article_order_list.connect_row_activated(clone!(
            @weak self.article_order_pop as article_order_pop,
            @weak self.article_order_label as article_order_label,
            @weak self.settings as settings,
            @strong sender => move |_list, row| {
            article_order_pop.popdown();
            let new_order = match row.get_index() {
                0 => ArticleOrder::NewestFirst,
                _ => ArticleOrder::OldestFirst,
            };
            article_order_label.set_label(new_order.to_str());
            if settings.write().set_article_list_order(new_order).is_ok() {
                Util::send(&sender, Action::UpdateArticleList);
            } else {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to set setting 'article order'.".to_owned()),
                );
            }
        }));

        self.article_order_event.connect_button_press_event(clone!(
            @weak self.article_order_pop as article_order_pop => @default-panic, move |_eventbox, event|
        {
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
        }));

        if let Some(listbox) = self.article_order_row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                listbox.connect_row_activated(
                    clone!(@weak self.article_order_pop as article_order_pop => move |_list, row| {
                        if let Some(name) = row.get_widget_name() {
                            if name == "article_order_row" {
                                article_order_pop.popup();
                            }
                        }
                    }),
                );
            }
        }

        self.article_theme_event.connect_button_press_event(clone!(
            @weak self.settings as settings,
            @weak self.article_theme_label as article_theme_label,
            @strong sender => @default-panic, move |eventbox, event|
        {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false);
                }
                _ => {}
            }

            let theme_chooser = ThemeChooser::new(eventbox, &sender, &settings);
            theme_chooser.widget().connect_closed(clone!(
                @weak settings,
                @weak article_theme_label,
                @strong sender => move |_pop|
            {
                article_theme_label.set_label(settings.read().get_article_view_theme().name());
                Util::send(&sender, Action::RedrawArticle);
            }));
            theme_chooser.widget().popup();

            Inhibit(false)
        }));

        if let Some(listbox) = self.article_theme_row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                listbox.connect_row_activated(clone!(
                    @weak self.article_theme_label as article_theme_label,
                    @weak self.article_theme_event as article_theme_event,
                    @weak self.settings as settings,
                    @strong sender => move |_list, row|
                {
                    if let Some(name) = row.get_widget_name() {
                        if name == "article_theme_row" {
                            let theme_chooser = ThemeChooser::new(&article_theme_event, &sender, &settings);
                            theme_chooser.widget().connect_closed(clone!(
                                @strong sender,
                                @weak settings => move |_pop|
                            {
                                article_theme_label.set_label(settings.read().get_article_view_theme().name());
                                Util::send(&sender, Action::RedrawArticle);
                            }));
                            theme_chooser.widget().popup();
                        }
                    }
                }));
            }
        }

        self.allow_selection_switch.connect_state_set(clone!(
            @weak self.settings as settings,
            @strong sender => @default-panic, move |_switch, is_set|
        {
            if settings.write().set_article_view_allow_select(is_set).is_ok() {
                Util::send(&sender, Action::RedrawArticle);
            } else {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to set setting 'allow article selection'.".to_owned()),
                );
            }
            Inhibit(false)
        }));

        self.font_button.connect_font_set(
            clone!(@weak self.settings as settings, @strong sender => move |button| {
                let font = match button.get_font() {
                    Some(font) => Some(font.to_string()),
                    None => None,
                };
                if settings.write().set_article_view_font(font).is_ok() {
                    Util::send(&sender, Action::RedrawArticle);
                } else {
                    Util::send(
                        &sender,
                        Action::ErrorSimpleMessage("Failed to set setting 'article font'.".to_owned()),
                    );
                }
            }),
        );

        self.use_system_font_switch.connect_state_set(clone!(
            @weak self.font_button as font_button,
            @weak self.font_row as font_row,
            @weak self.settings as settings,
            @strong sender => @default-panic, move |_switch, is_set|
        {
            let font = if is_set {
                None
            } else if let Some(font_name) = font_button.get_font() {
                Some(font_name.to_string())
            } else {
                None
            };
            font_button.set_sensitive(!is_set);
            font_row.set_sensitive(!is_set);
            if settings.write().set_article_view_font(font).is_ok() {
                Util::send(&sender, Action::RedrawArticle);
            } else {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to set setting 'use system font'.".to_owned()),
                );
            }
            Inhibit(false)
        }));
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
        let id = id.to_owned();

        if let Some(listbox) = row.get_parent() {
            if let Ok(listbox) = listbox.downcast::<ListBox>() {
                let info_text = row.get_title().expect(GTK_BUILDER_ERROR);
                listbox.connect_row_activated(clone!(
                    @weak self.widget as dialog,
                    @weak self.settings as settings,
                    @strong sender,
                    @strong id => move |_list, row|
                {
                    if let Some(name) = row.get_widget_name() {
                        if name.as_str() == row_name {
                            let editor = KeybindingEditor::new(&dialog, &info_text);
                            editor.widget().present();
                            editor.widget().connect_close(clone!(
                                @weak label,
                                @weak settings,
                                @strong sender,
                                @strong id => move |_dialog|
                            {
                                let _settings = settings.clone();
                                match &*editor.keybinding.read() {
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
                            }));
                        }
                    }
                }));
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
