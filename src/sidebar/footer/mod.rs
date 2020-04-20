use super::models::SidebarSelection;
use crate::app::Action;
use crate::main_window_state::MainWindowState;
use crate::util::{BuilderHelper, Util};
use glib::{clone, Sender};
use gtk::{Button, ButtonExt, WidgetExt};
use news_flash::models::PluginCapabilities;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug)]
pub struct SidebarFooter {
    pub add_button: Button,
    remove_button: Button,
    state: Arc<RwLock<MainWindowState>>,
    support_mutation: RwLock<bool>,
    sidebar_selection: Arc<RwLock<SidebarSelection>>,
}

impl SidebarFooter {
    pub fn new(
        builder: &BuilderHelper,
        state: &Arc<RwLock<MainWindowState>>,
        sender: &Sender<Action>,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
        sidebar_selection: &Arc<RwLock<SidebarSelection>>,
    ) -> Self {
        let add_button = builder.get::<Button>("add_button");
        let remove_button = builder.get::<Button>("remove_button");

        remove_button.connect_clicked(clone!(@strong sender => move |_button| {
            Util::send(&sender, Action::DeleteSidebarSelection);
        }));

        add_button.connect_clicked(clone!(@strong sender => move |_button| {
            Util::send(&sender, Action::AddDialog);
        }));

        let mut support_mutation = false;
        if let Some(features) = features.read().as_ref() {
            support_mutation = features.contains(PluginCapabilities::ADD_REMOVE_FEEDS);
        }

        SidebarFooter {
            add_button,
            remove_button,
            state: state.clone(),
            support_mutation: RwLock::new(support_mutation),
            sidebar_selection: sidebar_selection.clone(),
        }
    }

    pub fn update(&self) {
        self.add_button
            .set_sensitive(!self.state.read().get_offline() && *self.support_mutation.read());
        self.remove_button.set_sensitive(
            !self.state.read().get_offline()
                && *self.support_mutation.read()
                && *self.sidebar_selection.read() != SidebarSelection::All,
        );
    }

    pub fn update_features(&self, features: &Arc<RwLock<Option<PluginCapabilities>>>) {
        if let Some(features) = features.read().as_ref() {
            *self.support_mutation.write() = features.contains(PluginCapabilities::ADD_REMOVE_FEEDS);
            self.update();
        }
    }
}
