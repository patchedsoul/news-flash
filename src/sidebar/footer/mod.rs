use crate::app::Action;
use crate::main_window_state::MainWindowState;
use crate::util::{BuilderHelper, Util};
use glib::{clone, Sender};
use gtk::{Button, ButtonExt, WidgetExt};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SidebarFooter {
    add_button: Button,
    remove_button: Button,
    state: Arc<RwLock<MainWindowState>>,
}

impl SidebarFooter {
    pub fn new(builder: &BuilderHelper, state: &Arc<RwLock<MainWindowState>>, sender: &Sender<Action>) -> Self {
        let add_button = builder.get::<Button>("add_button");
        let remove_button = builder.get::<Button>("remove_button");

        remove_button.connect_clicked(clone!(@strong sender => move |_button| {
            Util::send(&sender, Action::DeleteSidebarSelection);
        }));

        add_button.connect_clicked(clone!(@strong sender => move |_button| {
            Util::send(&sender, Action::AddDialog);
        }));

        SidebarFooter {
            add_button,
            remove_button,
            state: state.clone(),
        }
    }

    pub fn set_remove_button_sensitive(&self, sensitive: bool) {
        if !self.state.read().get_offline() {
            self.remove_button.set_sensitive(sensitive);
        }
    }

    pub fn get_add_button(&self) -> Button {
        self.add_button.clone()
    }

    pub fn set_offline(&self, offline: bool) {
        self.add_button.set_sensitive(!offline);
        self.remove_button.set_sensitive(!offline);
    }
}
