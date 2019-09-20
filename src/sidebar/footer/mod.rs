use crate::util::{BuilderHelper, GtkUtil};
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
            GtkUtil::execute_action(button, "delete-selection", None);
        });

        add_button.connect_clicked(|button| {
            GtkUtil::execute_action(button, "add-feed", None);
        });

        SidebarFooter {
            add_button,
            remove_button,
        }
    }

    pub fn set_remove_button_sensitive(&self, sensitive: bool) {
        self.remove_button.set_sensitive(sensitive);
    }

    pub fn get_add_button(&self) -> Button {
        self.add_button.clone()
    }
}
