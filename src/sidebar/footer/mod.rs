use crate::util::{BuilderHelper, GtkUtil};
use gio::{ActionExt, ActionMapExt};
use gtk::{Button, ButtonExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct SidebarFooter {
    add_button: Button,
    remove_button: Button,
}

impl SidebarFooter {
    pub fn new(builder: &BuilderHelper) -> Self {
        let add_button = builder.get::<Button>("add_button");
        let remove_button = builder.get::<Button>("remove_button");

        remove_button.connect_clicked(|button| {
            if let Ok(main_window) = GtkUtil::get_main_window(button) {
                if let Some(action) = main_window.lookup_action("delete-selection-action") {
                    action.activate(None);
                }
            }
        });

        SidebarFooter {
            add_button,
            remove_button,
        }
    }

    pub fn set_remove_button_sensitive(&self, sensitive: bool) {
        self.remove_button.set_sensitive(sensitive);
    }
}
