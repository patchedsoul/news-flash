mod models;

pub use models::UndoAction;
use crate::util::{BuilderHelper, GtkUtil, GtkHandle};
use crate::gtk_handle;
use gtk::{InfoBar};
use gio::{ActionExt, ActionMapExt};
use glib::{Variant};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct UndoBar {
    widget: InfoBar,
    current_action: Option<UndoAction>,
    timeout: GtkHandle<Option<u32>>,
}

impl UndoBar {
    pub fn new(builder: &BuilderHelper) -> Self {
        UndoBar {
            widget: builder.get::<InfoBar>("undo_bar"),
            current_action: None,
            timeout: gtk_handle!(None),
        }
    }

    fn execute_action(action: UndoAction, bar: InfoBar) {
        match action {
            UndoAction::DeleteFeed(feed_id) => {
                if let Ok(main_window) = GtkUtil::get_main_window(&bar) {
                    if let Some(action) = main_window.lookup_action("delete-feed") {
                        let variant = Variant::from(feed_id.to_str());
                        action.activate(Some(&variant));
                    }
                }
            },
            UndoAction::DeleteCategory(category_id) => {
                if let Ok(main_window) = GtkUtil::get_main_window(&bar) {
                    if let Some(action) = main_window.lookup_action("delete-category") {
                        let variant = Variant::from(category_id.to_str());
                        action.activate(Some(&variant));
                    }
                }
            },
        }
    }

    pub fn add_action(&self, action: UndoAction) {
        // check for enqued action
        // execute enqued action immediately
        // enqueue new action

        // update lists
        if let Ok(main_window) = GtkUtil::get_main_window(&self.widget) {
            if let Some(action) = main_window.lookup_action("update-sidebar") {
                action.activate(None);
            }
            if let Some(action) = main_window.lookup_action("update-article-list") {
                action.activate(None);
            }
        }
    }
}