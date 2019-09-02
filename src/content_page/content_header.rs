use super::header_selection::HeaderSelection;
use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
use gdk::EventType;
use gio::{ActionExt, ActionMapExt, Menu, MenuItem};
use glib::{translate::ToGlib, Variant};
use gtk::{
    Button, ButtonExt, Continue, EntryExt, Inhibit, MenuButton, MenuButtonExt, SearchEntry, SearchEntryExt, Stack,
    StackExt, ToggleButton, ToggleButtonExt, WidgetExt,
};
use libhandy::{SearchBar, SearchBarExt};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ContentHeader {
    update_stack: Stack,
    update_button: Button,
    search_button: ToggleButton,
    search_entry: SearchEntry,
    all_button: ToggleButton,
    unread_button: ToggleButton,
    marked_button: ToggleButton,
    more_actions_button: MenuButton,
    mode_switch_stack: Stack,
}

impl ContentHeader {
    pub fn new(builder: &BuilderHelper) -> Self {
        let all_button = builder.get::<ToggleButton>("all_button");
        let unread_button = builder.get::<ToggleButton>("unread_button");
        let marked_button = builder.get::<ToggleButton>("marked_button");
        let update_button = builder.get::<Button>("update_button");
        let update_stack = builder.get::<Stack>("update_stack");
        let menu_button = builder.get::<MenuButton>("menu_button");
        let more_actions_button = builder.get::<MenuButton>("more_actions_button");
        let search_button = builder.get::<ToggleButton>("search_button");
        let search_bar = builder.get::<SearchBar>("search_bar");
        let search_entry = builder.get::<SearchEntry>("search_entry");
        let mode_button = builder.get::<MenuButton>("mode_switch_button");
        let mode_switch_stack = builder.get::<Stack>("mode_switch_stack");

        let linked_button_timeout: GtkHandle<Option<u32>> = gtk_handle!(None);
        let header_selection = gtk_handle!(HeaderSelection::All);

        Self::setup_linked_button(
            &all_button,
            &unread_button,
            &marked_button,
            &header_selection,
            &linked_button_timeout,
            HeaderSelection::All,
        );
        Self::setup_linked_button(
            &unread_button,
            &all_button,
            &marked_button,
            &header_selection,
            &linked_button_timeout,
            HeaderSelection::Unread,
        );
        Self::setup_linked_button(
            &marked_button,
            &unread_button,
            &all_button,
            &header_selection,
            &linked_button_timeout,
            HeaderSelection::Marked,
        );
        Self::setup_update_button(&update_button, &update_stack);
        Self::setup_search_button(&search_button, &search_bar);
        Self::setup_search_bar(&search_bar, &search_button, &search_entry);
        Self::setup_search_entry(&search_entry);

        Self::setup_menu_button(&menu_button);
        Self::setup_mode_button(&mode_button);
        Self::setup_more_actions_button(&more_actions_button);

        ContentHeader {
            update_stack,
            update_button,
            search_button,
            search_entry,
            all_button,
            unread_button,
            marked_button,
            more_actions_button,
            mode_switch_stack,
        }
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
        button: &ToggleButton,
        other_button_1: &ToggleButton,
        other_button_2: &ToggleButton,
        header_selection: &GtkHandle<HeaderSelection>,
        linked_button_timeout: &GtkHandle<Option<u32>>,
        mode: HeaderSelection,
    ) {
        let other_button_1_1 = other_button_1.clone();
        let other_button_2_1 = other_button_2.clone();
        button.connect_button_press_event(move |button, event| {
            if button.get_active() || event.get_button() != 1 {
                return Inhibit(true);
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(true),
            }
            other_button_1_1.set_active(false);
            other_button_2_1.set_active(false);
            Inhibit(false)
        });

        let header_selection = header_selection.clone();
        let linked_button_timeout = linked_button_timeout.clone();
        button.connect_toggled(move |button| {
            if !button.get_active() {
                // ignore deactivating toggle-button
                return;
            }

            *header_selection.borrow_mut() = mode.clone();

            if linked_button_timeout.borrow().is_some() {
                return;
            }

            Self::linked_button_toggled(button, &header_selection, &linked_button_timeout);
        });
    }

    fn linked_button_toggled(
        button: &ToggleButton,
        header_selection: &GtkHandle<HeaderSelection>,
        linked_button_timeout: &GtkHandle<Option<u32>>,
    ) {
        if let Ok(main_window) = GtkUtil::get_main_window(button) {
            if let Some(action) = main_window.lookup_action("headerbar-selection") {
                if let Ok(json) = serde_json::to_string(&*header_selection.borrow()) {
                    let json = Variant::from(&json);
                    action.activate(Some(&json));
                }
            }
        }

        if linked_button_timeout.borrow().is_some() {
            return;
        }

        let toggle_button = button.clone();
        let mode_before_cooldown = (*header_selection.borrow()).clone();
        let header_selection = header_selection.clone();
        let linked_button_timeout_clone = linked_button_timeout.clone();
        *linked_button_timeout.borrow_mut() = Some(
            gtk::timeout_add(250, move || {
                *linked_button_timeout_clone.borrow_mut() = None;
                if mode_before_cooldown != *header_selection.borrow() {
                    Self::linked_button_toggled(&toggle_button, &header_selection, &linked_button_timeout_clone);
                }
                Continue(false)
            })
            .to_glib(),
        );
    }

    fn setup_update_button(button: &Button, stack: &Stack) {
        let stack = stack.clone();
        button.connect_clicked(move |button| {
            button.set_sensitive(false);
            stack.set_visible_child_name("spinner");

            if let Ok(main_window) = GtkUtil::get_main_window(button) {
                if let Some(action) = main_window.lookup_action("sync") {
                    action.activate(None);
                }
            }
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
        search_bar.connect_property_search_mode_enabled_notify(move |bar| {
            if !bar.get_search_mode() {
                search_button.set_active(false);
            }
        });
    }

    fn setup_search_entry(search_entry: &SearchEntry) {
        search_entry.connect_search_changed(|search_entry| {
            if let Ok(main_window) = GtkUtil::get_main_window(search_entry) {
                if let Some(action) = main_window.lookup_action("search-term") {
                    if let Some(text) = search_entry.get_text() {
                        let search_term = Variant::from(text.as_str());
                        action.activate(Some(&search_term));
                    }
                }
            }
        });
    }

    fn setup_menu_button(button: &MenuButton) {
        let about_model = Menu::new();
        about_model.append(Some("Shortcuts"), Some("win.shortcuts"));
        about_model.append(Some("About"), Some("win.about"));

        let im_export_model = Menu::new();
        im_export_model.append(Some("Import OPML"), Some("win.import"));
        im_export_model.append(Some("Export OPML"), Some("win.export"));

        let main_model = Menu::new();
        main_model.append(Some("Settings"), Some("win.settings"));
        main_model.append_section(Some(""), &im_export_model);
        main_model.append_section(Some(""), &about_model);

        button.set_menu_model(Some(&main_model));
    }

    fn setup_mode_button(button: &MenuButton) {
        let model = Menu::new();
        if let Ok(json) = serde_json::to_string(&HeaderSelection::All) {
            let variant = Variant::from(&json);
            let all_item = MenuItem::new(Some("All"), None);
            all_item.set_action_and_target_value(Some("win.headerbar-selection"), Some(&variant));
            model.append_item(&all_item);
        }

        if let Ok(json) = serde_json::to_string(&HeaderSelection::Unread) {
            let variant = Variant::from(&json);
            let unread_item = MenuItem::new(Some("Unread"), None);
            unread_item.set_action_and_target_value(Some("win.headerbar-selection"), Some(&variant));
            model.append_item(&unread_item);
        }

        if let Ok(json) = serde_json::to_string(&HeaderSelection::Marked) {
            let variant = Variant::from(&json);
            let marked_item = MenuItem::new(Some("Starred"), None);
            marked_item.set_action_and_target_value(Some("win.headerbar-selection"), Some(&variant));
            model.append_item(&marked_item);
        }

        button.set_menu_model(Some(&model));
    }

    fn setup_more_actions_button(button: &MenuButton) {
        let model = Menu::new();
        model.append(Some("Export Article"), Some("win.export-article"));
        model.append(Some("Close Article"), Some("win.close-article"));
        button.set_menu_model(Some(&model));
        button.set_sensitive(false);
    }

    pub fn set_article_header_sensitive(&self, sensitive: bool) {
        self.more_actions_button.set_sensitive(sensitive);
    }
}
