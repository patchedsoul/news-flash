use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use crate::gtk_util::GtkUtil;
use gtk::{
    Builder,
    PanedExt,
    ToggleButton,
    ToggleButtonExt,
    WidgetExt,
};
use glib::{
    Variant,
    signal::Inhibit,
};
use gio::{
    ActionExt,
    ActionMapExt,
};

pub struct ContentHeader {
    header: gtk::Paned,
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

        Self::setup_linked_button(&all_button, &unread_button, &marked_button);
        Self::setup_linked_button(&unread_button, &all_button, &marked_button);
        Self::setup_linked_button(&marked_button, &unread_button, &all_button);

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
        })
    }

    pub fn widget(&self) -> gtk::Paned {
        self.header.clone()
    }

    pub fn set_paned(&self, pos: i32) {
        self.header.set_position(pos);
    }

    fn setup_linked_button(button: &ToggleButton, other_button_1: &ToggleButton, other_button_2: &ToggleButton) {
        let button_clone = button.clone();
        let other_button_1 = other_button_1.clone();
        let other_button_2 = other_button_2.clone();
        button.connect_button_press_event(move |_button, _event| {
            if button_clone.get_active() {
                // ignore deactivating toggle-button
                return Inhibit(true)
            }
            other_button_1.set_active(false);
            other_button_2.set_active(false);
            Inhibit(false)
        });
    }
}