mod models;

use crate::util::{BuilderHelper, GtkUtil};
use models::UndoAction;
use gtk::{InfoBar};
use gio::{ActionExt, ActionMapExt};
use glib::{Variant};

#[derive(Clone, Debug)]
pub struct UndoBar {
    widget: InfoBar,
    current_action: Option<UndoAction>,
}

impl UndoBar {
    pub fn new(builder: &BuilderHelper) -> Self {
        UndoBar {
            widget: builder.get::<InfoBar>("undo_bar"),
            current_action: None,
        }
    }

    fn execute_action(&self) {
        if let Some(current_action) = &self.current_action {
            match current_action {
                UndoAction::DeleteFeed(feed_id) => {
                    if let Ok(main_window) = GtkUtil::get_main_window(&self.widget) {
                        if let Some(action) = main_window.lookup_action("delete-feed") {
                            let variant = Variant::from(feed_id.to_str());
                            action.activate(Some(&variant));
                        }
                    }
                },
                UndoAction::DeleteCategory(id) => {
                    if let Ok(main_window) = GtkUtil::get_main_window(&self.widget) {
                        if let Some(action) = main_window.lookup_action("delete-category") {
                            action.activate(None);
                        }
                    }
                },
            }
        }
    }
}