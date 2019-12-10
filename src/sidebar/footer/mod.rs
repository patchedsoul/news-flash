use crate::app::Action;
use crate::util::{BuilderHelper, GtkUtil, Util};
use glib::Sender;
use gtk::{Button, ButtonExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct SidebarFooter {
    add_button: Button,
    remove_button: Button,
}

impl SidebarFooter {
    pub fn new(builder: &BuilderHelper, sender: &Sender<Action>) -> Self {
        let add_button = builder.get::<Button>("add_button");
        let remove_button = builder.get::<Button>("remove_button");

        remove_button.connect_clicked(|button| {
            GtkUtil::execute_action(button, "delete-selection", None);
        });

        let sender = sender.clone();
        add_button.connect_clicked(move |_button| {
            Util::send(&sender, Action::AddFeedDialog);
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
