use super::header_selection::HeaderSelection;
use crate::app::Action;
use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil, Util};
use gio::{ActionMapExt, Menu, MenuItem, SimpleAction};
use glib::{object::Cast, translate::ToGlib, Sender};
use gtk::{
    Button, ButtonExt, Continue, EntryExt, Inhibit, MenuButton, MenuButtonExt, Popover, PopoverExt, SearchEntry,
    SearchEntryExt, Stack, StackExt, ToggleButton, ToggleButtonExt, WidgetExt,
};
use libhandy::{SearchBar, SearchBarExt};
use news_flash::models::{FatArticle, Marked, Read};
use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ContentHeader {
    sender: Sender<Action>,
    update_stack: Stack,
    update_button: Button,
    offline_button: Button,
    offline_stack: Stack,
    offline_popover: Popover,
    search_button: ToggleButton,
    search_entry: SearchEntry,
    mark_all_read_button: Button,
    mark_all_read_stack: Stack,
    all_button: ToggleButton,
    unread_button: ToggleButton,
    marked_button: ToggleButton,
    more_actions_button: MenuButton,
    more_actions_stack: Stack,
    mode_switch_stack: Stack,
    mark_article_button: ToggleButton,
    mark_article_read_button: ToggleButton,
    mark_article_stack: Stack,
    mark_article_read_stack: Stack,
    mark_article_event: RwLock<Option<u64>>,
    mark_article_read_event: RwLock<Option<u64>>,
}

impl ContentHeader {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let all_button = builder.get::<ToggleButton>("all_button");
        let unread_button = builder.get::<ToggleButton>("unread_button");
        let marked_button = builder.get::<ToggleButton>("marked_button");
        let update_button = builder.get::<Button>("update_button");
        let update_stack = builder.get::<Stack>("update_stack");
        let offline_button = builder.get::<Button>("offline_status_button");
        let offline_stack = builder.get::<Stack>("offline_status_stack");
        let offline_popover = builder.get::<Popover>("offline_popover");
        let menu_button = builder.get::<MenuButton>("menu_button");
        let more_actions_button = builder.get::<MenuButton>("more_actions_button");
        let more_actions_stack = builder.get::<Stack>("more_actions_stack");
        let search_button = builder.get::<ToggleButton>("search_button");
        let search_bar = builder.get::<SearchBar>("search_bar");
        let search_entry = builder.get::<SearchEntry>("search_entry");
        let mode_button = builder.get::<MenuButton>("mode_switch_button");
        let mode_switch_stack = builder.get::<Stack>("mode_switch_stack");
        let mark_all_read_button = builder.get::<Button>("mark_all_button");
        let mark_all_read_stack = builder.get::<Stack>("mark_all_stack");
        let mark_article_button = builder.get::<ToggleButton>("mark_article_button");
        let mark_article_read_button = builder.get::<ToggleButton>("mark_article_read_button");
        let mark_article_stack = builder.get::<Stack>("mark_article_stack");
        let mark_article_read_stack = builder.get::<Stack>("mark_article_read_stack");

        let sender_clone = sender.clone();
        let mark_all_read_stack_clone = mark_all_read_stack.clone();
        mark_all_read_button.connect_clicked(move |button| {
            button.set_sensitive(false);
            mark_all_read_stack_clone.set_visible_child_name("spinner");
            Util::send(&sender_clone, Action::SetSidebarRead);
        });

        let linked_button_timeout: GtkHandle<Option<u32>> = gtk_handle!(None);
        let header_selection = gtk_handle!(HeaderSelection::All);

        Self::setup_linked_button(
            &sender,
            &all_button,
            &unread_button,
            &marked_button,
            &header_selection,
            &linked_button_timeout,
            HeaderSelection::All,
        );
        Self::setup_linked_button(
            &sender,
            &unread_button,
            &all_button,
            &marked_button,
            &header_selection,
            &linked_button_timeout,
            HeaderSelection::Unread,
        );
        Self::setup_linked_button(
            &sender,
            &marked_button,
            &unread_button,
            &all_button,
            &header_selection,
            &linked_button_timeout,
            HeaderSelection::Marked,
        );
        Self::setup_update_button(&update_button, &sender);
        Self::setup_search_button(&search_button, &search_bar);
        Self::setup_search_bar(&search_bar, &search_button, &search_entry);
        Self::setup_search_entry(&search_entry, &sender);

        Self::setup_menu_button(&menu_button, &sender);
        Self::setup_mode_button(&mode_button, &sender);
        Self::setup_more_actions_button(&more_actions_button, &sender);

        let header = ContentHeader {
            sender,
            update_stack,
            update_button,
            offline_button,
            offline_stack,
            offline_popover,
            search_button,
            search_entry,
            mark_all_read_button,
            mark_all_read_stack,
            all_button,
            unread_button,
            marked_button,
            more_actions_button,
            more_actions_stack,
            mode_switch_stack,
            mark_article_button,
            mark_article_read_button,
            mark_article_stack,
            mark_article_read_stack,
            mark_article_event: RwLock::new(None),
            mark_article_read_event: RwLock::new(None),
        };

        header.show_article(None);
        header
    }

    pub fn start_sync(&self) {
        self.update_button.set_sensitive(false);
        self.update_stack.set_visible_child_name("spinner");
    }

    pub fn finish_sync(&self) {
        self.update_button.set_sensitive(true);
        self.update_stack.set_visible_child_name("icon");
    }

    pub fn is_search_focused(&self) -> bool {
        self.search_button.get_active() && self.search_entry.has_focus()
    }

    pub fn focus_search(&self) {
        // shortcuts ignored when focues -> no need to hide seach bar on keybind (ESC still works)
        self.search_button.set_active(true);
        self.search_entry.grab_focus();
    }

    pub fn select_all_button(&self) {
        self.all_button.set_active(true);
        self.unread_button.set_active(false);
        self.marked_button.set_active(false);
        self.mode_switch_stack.set_visible_child_name("all");
    }

    pub fn select_unread_button(&self) {
        self.unread_button.set_active(true);
        self.all_button.set_active(false);
        self.marked_button.set_active(false);
        self.mode_switch_stack.set_visible_child_name("unread");
    }

    pub fn select_marked_button(&self) {
        self.marked_button.set_active(true);
        self.all_button.set_active(false);
        self.unread_button.set_active(false);
        self.mode_switch_stack.set_visible_child_name("marked");
    }

    fn setup_linked_button(
        sender: &Sender<Action>,
        button: &ToggleButton,
        other_button_1: &ToggleButton,
        other_button_2: &ToggleButton,
        header_selection: &GtkHandle<HeaderSelection>,
        linked_button_timeout: &GtkHandle<Option<u32>>,
        mode: HeaderSelection,
    ) {
        button.connect_button_press_event(|button, _event| {
            let toggle_button = button
                .clone()
                .downcast::<ToggleButton>()
                .expect("Failed to cast to ToggleButton");

            if toggle_button.get_active() {
                return Inhibit(true);
            }

            Inhibit(false)
        });

        let other_button_1 = other_button_1.clone();
        let other_button_2 = other_button_2.clone();
        let header_selection = header_selection.clone();
        let linked_button_timeout = linked_button_timeout.clone();
        let sender = sender.clone();
        button.connect_toggled(move |button| {
            if !button.get_active() {
                // ignore deactivating toggle-button
                return;
            }

            other_button_1.set_active(false);
            other_button_2.set_active(false);

            *header_selection.borrow_mut() = mode.clone();

            if linked_button_timeout.borrow().is_some() {
                return;
            }

            Self::linked_button_toggled(&sender, button, &header_selection, &linked_button_timeout);
        });
    }

    fn linked_button_toggled(
        sender: &Sender<Action>,
        button: &ToggleButton,
        header_selection: &GtkHandle<HeaderSelection>,
        linked_button_timeout: &GtkHandle<Option<u32>>,
    ) {
        Util::send(sender, Action::HeaderSelection((*header_selection.borrow()).clone()));

        if linked_button_timeout.borrow().is_some() {
            return;
        }

        let toggle_button = button.clone();
        let mode_before_cooldown = (*header_selection.borrow()).clone();
        let header_selection = header_selection.clone();
        let linked_button_timeout_clone = linked_button_timeout.clone();
        let sender_clone = sender.clone();
        *linked_button_timeout.borrow_mut() = Some(
            gtk::timeout_add(250, move || {
                *linked_button_timeout_clone.borrow_mut() = None;
                if mode_before_cooldown != *header_selection.borrow() {
                    Self::linked_button_toggled(
                        &sender_clone,
                        &toggle_button,
                        &header_selection,
                        &linked_button_timeout_clone,
                    );
                }
                Continue(false)
            })
            .to_glib(),
        );
    }

    fn setup_update_button(button: &Button, sender: &Sender<Action>) {
        let sender = sender.clone();
        button.connect_clicked(move |_button| {
            Util::send(&sender, Action::Sync);
        });
    }

    fn setup_search_button(search_button: &ToggleButton, search_bar: &SearchBar) {
        let search_bar = search_bar.clone();
        search_button.connect_toggled(move |button| {
            if button.get_active() {
                search_bar.set_search_mode(true);
            } else {
                search_bar.set_search_mode(false);
            }
        });
    }

    fn setup_search_bar(search_bar: &SearchBar, search_button: &ToggleButton, search_entry: &SearchEntry) {
        search_bar.connect_entry(search_entry);
        let search_button = search_button.clone();
        search_bar.connect_property_search_mode_enabled_notify(move |search_bar| {
            if !search_bar.get_search_mode() {
                search_button.set_active(false);
            }
        });
    }

    fn setup_search_entry(search_entry: &SearchEntry, sender: &Sender<Action>) {
        let sender = sender.clone();
        search_entry.connect_search_changed(move |search_entry| {
            if let Some(text) = search_entry.get_text() {
                Util::send(&sender, Action::SearchTerm(text.as_str().to_owned()));
            }
        });
    }

    fn setup_menu_button(button: &MenuButton, sender: &Sender<Action>) {
        let sender_clone = sender.clone();
        let show_shortcut_window_action = SimpleAction::new("shortcut-window", None);
        show_shortcut_window_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::ShowShortcutWindow);
        });

        let sender_clone = sender.clone();
        let show_about_window_action = SimpleAction::new("about-window", None);
        show_about_window_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::ShowAboutWindow);
        });

        let sender_clone = sender.clone();
        let settings_window_action = SimpleAction::new("settings", None);
        settings_window_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::ShowSettingsWindow);
        });

        let sender_clone = sender.clone();
        let quit_action = SimpleAction::new("quit-application", None);
        quit_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::QueueQuit);
        });

        let sender_clone = sender.clone();
        let export_opml_action = SimpleAction::new("export-opml", None);
        export_opml_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::ExportOpml);
        });

        let sender_clone = sender.clone();
        let relogin_action = SimpleAction::new("relogin", None);
        relogin_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::RetryLogin);
        });

        let sender_clone = sender.clone();
        let reset_account_action = SimpleAction::new("reset-account", None);
        reset_account_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::ShowResetPage);
        });

        if let Ok(main_window) = GtkUtil::get_main_window(button) {
            main_window.add_action(&show_shortcut_window_action);
            main_window.add_action(&show_about_window_action);
            main_window.add_action(&settings_window_action);
            main_window.add_action(&quit_action);
            main_window.add_action(&export_opml_action);
            main_window.add_action(&relogin_action);
            main_window.add_action(&reset_account_action);
        }

        let about_model = Menu::new();
        about_model.append(Some("Shortcuts"), Some("win.shortcut-window"));
        about_model.append(Some("About"), Some("win.about-window"));
        about_model.append(Some("Quit"), Some("win.quit-application"));

        let im_export_model = Menu::new();
        im_export_model.append(Some("Import OPML"), Some("win.import"));
        im_export_model.append(Some("Export OPML"), Some("win.export-opml"));

        let account_model = Menu::new();
        account_model.append(Some("Update Login"), Some("win.relogin"));
        account_model.append(Some("Reset Login"), Some("win.reset-account"));

        let main_model = Menu::new();
        main_model.append(Some("Settings"), Some("win.settings"));
        main_model.append_section(Some(""), &account_model);
        main_model.append_section(Some(""), &im_export_model);
        main_model.append_section(Some(""), &about_model);

        button.set_menu_model(Some(&main_model));
    }

    fn setup_mode_button(button: &MenuButton, sender: &Sender<Action>) {
        let model = Menu::new();

        let sender_clone = sender.clone();
        let headerbar_selection_all_action = SimpleAction::new("headerbar-selection-all", None);
        headerbar_selection_all_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::HeaderSelection(HeaderSelection::All));
        });
        let all_item = MenuItem::new(Some("All"), None);
        all_item.set_action_and_target_value(Some("win.headerbar-selection-all"), None);
        model.append_item(&all_item);

        let sender_clone = sender.clone();
        let headerbar_selection_unread_action = SimpleAction::new("headerbar-selection-unread", None);
        headerbar_selection_unread_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::HeaderSelection(HeaderSelection::Unread));
        });
        let unread_item = MenuItem::new(Some("Unread"), None);
        unread_item.set_action_and_target_value(Some("win.headerbar-selection-unread"), None);
        model.append_item(&unread_item);

        let sender_clone = sender.clone();
        let headerbar_selection_marked_action = SimpleAction::new("headerbar-selection-marked", None);
        headerbar_selection_marked_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::HeaderSelection(HeaderSelection::Marked));
        });
        let marked_item = MenuItem::new(Some("Starred"), None);
        marked_item.set_action_and_target_value(Some("win.headerbar-selection-marked"), None);
        model.append_item(&marked_item);

        if let Ok(main_window) = GtkUtil::get_main_window(button) {
            main_window.add_action(&headerbar_selection_all_action);
            main_window.add_action(&headerbar_selection_unread_action);
            main_window.add_action(&headerbar_selection_marked_action);
        }

        button.set_menu_model(Some(&model));
    }

    fn setup_more_actions_button(button: &MenuButton, sender: &Sender<Action>) {
        let sender_clone = sender.clone();
        let close_article_action = SimpleAction::new("close-article", None);
        close_article_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::CloseArticle);
        });

        let sender_clone = sender.clone();
        let export_article_action = SimpleAction::new("export-article", None);
        export_article_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::ExportArticle);
        });

        let sender_clone = sender.clone();
        let grab_article_content_action = SimpleAction::new("grab-article-content", None);
        grab_article_content_action.connect_activate(move |_action, _parameter| {
            Util::send(&sender_clone, Action::StartGrabArticleContent);
        });

        if let Ok(main_window) = GtkUtil::get_main_window(button) {
            main_window.add_action(&close_article_action);
            main_window.add_action(&export_article_action);
            main_window.add_action(&grab_article_content_action);
        }

        let model = Menu::new();
        model.append(Some("Grab full content"), Some("win.grab-article-content"));
        model.append(Some("Export Article"), Some("win.export-article"));
        model.append(Some("Close Article"), Some("win.close-article"));
        button.set_menu_model(Some(&model));
        button.set_sensitive(false);
    }

    fn unread_button_state(article: Option<&FatArticle>) -> (&str, bool) {
        match article {
            Some(article) => match article.unread {
                Read::Read => ("read", false),
                Read::Unread => ("unread", true),
            },
            None => ("read", false),
        }
    }

    fn marked_button_state(article: Option<&FatArticle>) -> (&str, bool) {
        match article {
            Some(article) => match article.marked {
                Marked::Marked => ("marked", true),
                Marked::Unmarked => ("unmarked", false),
            },
            None => ("unmarked", false),
        }
    }

    pub fn show_article(&self, article: Option<&FatArticle>) {
        let sensitive = article.is_some();

        let (unread_icon, unread_active) = Self::unread_button_state(article);
        let (marked_icon, marked_active) = Self::marked_button_state(article);

        self.mark_article_stack.set_visible_child_name(marked_icon);
        self.mark_article_read_stack.set_visible_child_name(unread_icon);

        GtkUtil::disconnect_signal(*self.mark_article_read_event.read(), &self.mark_article_read_button);
        GtkUtil::disconnect_signal(*self.mark_article_event.read(), &self.mark_article_button);

        self.mark_article_button.set_active(marked_active);
        self.mark_article_read_button.set_active(unread_active);

        let sender_clone = self.sender.clone();
        let toggle_stack = self.mark_article_stack.clone();
        self.mark_article_event.write().replace(
            self.mark_article_button
                .connect_toggled(move |toggle_button| {
                    if toggle_button.get_active() {
                        toggle_stack.set_visible_child_name("marked");
                    } else {
                        toggle_stack.set_visible_child_name("unmarked");
                    }
                    Util::send(&sender_clone, Action::ToggleArticleMarked);
                })
                .to_glib(),
        );

        let sender_clone = self.sender.clone();
        let toggle_stack = self.mark_article_read_stack.clone();
        self.mark_article_read_event.write().replace(
            self.mark_article_read_button
                .connect_toggled(move |toggle_button| {
                    if toggle_button.get_active() {
                        toggle_stack.set_visible_child_name("unread");
                    } else {
                        toggle_stack.set_visible_child_name("read");
                    }
                    Util::send(&sender_clone, Action::ToggleArticleRead);
                })
                .to_glib(),
        );

        self.more_actions_button.set_sensitive(sensitive);
        self.mark_article_button.set_sensitive(sensitive);
        self.mark_article_read_button.set_sensitive(sensitive);
    }

    pub fn start_more_actions_spinner(&self) {
        self.more_actions_button.set_sensitive(false);
        self.more_actions_stack.set_visible_child_name("spinner");
    }

    pub fn stop_more_actions_spinner(&self) {
        self.more_actions_button.set_sensitive(true);
        self.more_actions_stack.set_visible_child_name("image");
    }

    pub fn finish_mark_all_read(&self) {
        self.mark_all_read_button.set_sensitive(true);
        self.mark_all_read_stack.set_visible_child_name("image");
    }

    pub fn set_offline(&self, offline: bool) {
        self.offline_button.set_visible(offline);
        self.update_button.set_visible(!offline);
        self.mark_all_read_button.set_visible(!offline);
        if offline {
            self.offline_popover.popup();
        } else {
            self.offline_popover.popdown();
        }
    }
}
