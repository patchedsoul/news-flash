use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use crate::util::GtkUtil;
use gtk::{
    Builder,
    PanedExt,
    ToggleButton,
    ToggleButtonExt,
    WidgetExt,
    ButtonExt,
    StackExt,
    EntryExt,
    SearchEntryExt,
};
use gdk::{
    EventType,
};
use glib::{
    Variant,
    signal::Inhibit,
};
use gio::{
    ActionExt,
    ActionMapExt,
};
use super::header_selection::HeaderSelection;

pub struct ContentHeader {
    header: gtk::Paned,
    stack: gtk::Stack,
    update_button: gtk::Button,
}

impl ContentHeader {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/content_page_header.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let header : gtk::Paned = builder.get_object("content_header").ok_or(format_err!("some err"))?;
        let all_button : ToggleButton = builder.get_object("all_button").ok_or(format_err!("some err"))?;
        let unread_button : ToggleButton = builder.get_object("unread_button").ok_or(format_err!("some err"))?;
        let marked_button : ToggleButton = builder.get_object("marked_button").ok_or(format_err!("some err"))?;
        let update_button : gtk::Button = builder.get_object("update_button").ok_or(format_err!("some err"))?;
        let update_stack : gtk::Stack = builder.get_object("update_stack").ok_or(format_err!("some err"))?;
        let search_entry : gtk::SearchEntry = builder.get_object("search_entry").ok_or(format_err!("some err"))?;
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

        Self::setup_linked_button(&all_button, &unread_button, &marked_button, HeaderSelection::All);
        Self::setup_linked_button(&unread_button, &all_button, &marked_button, HeaderSelection::Unread);
        Self::setup_linked_button(&marked_button, &unread_button, &all_button, HeaderSelection::Marked);
        Self::setup_update_button(&update_button, &update_stack);

        header.connect_property_position_notify(|paned| {
            if let Ok(main_window) = GtkUtil::get_main_window(paned) {
                if let Some(action) = main_window.lookup_action("sync-paned") {
                    let pos = Variant::from(&paned.get_position());
                    action.activate(Some(&pos));
                }
            }
        });

        Ok(ContentHeader {
            header: header,
            stack: update_stack,
            update_button: update_button,
        })
    }

    pub fn widget(&self) -> gtk::Paned {
        self.header.clone()
    }

    pub fn set_paned(&self, pos: i32) {
        self.header.set_position(pos);
    }

    pub fn finish_sync(&self) {
        self.update_button.set_sensitive(true);
        self.stack.set_visible_child_name("icon");
    }

    fn setup_linked_button(button: &ToggleButton, other_button_1: &ToggleButton, other_button_2: &ToggleButton, mode: HeaderSelection) {
        let button_clone = button.clone();
        let other_button_1 = other_button_1.clone();
        let other_button_2 = other_button_2.clone();
        button.connect_button_press_event(move |button, event| {
            if event.get_button() != 1 {
                return gtk::Inhibit(false)
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }
            if button_clone.get_active() {
                // ignore deactivating toggle-button
                return Inhibit(true)
            }
            other_button_1.set_active(false);
            other_button_2.set_active(false);
            if let Ok(main_window) = GtkUtil::get_main_window(button) {
                if let Some(action) = main_window.lookup_action("headerbar-selection") {
                    if let Ok(json) = serde_json::to_string(&mode) {
                        let json = Variant::from(&json);
                        action.activate(Some(&json));
                    }
                }
            }
            Inhibit(false)
        });
    }

    fn setup_update_button(button: &gtk::Button, stack: &gtk::Stack) {
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
}