mod models;

use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
use gio::{ActionExt, ActionMapExt};
use glib::{translate::ToGlib, Variant};
use gtk::{Button, ButtonExt, Continue, InfoBar, InfoBarExt, Label, LabelExt};
pub use models::{UndoAction, UndoActionType};
use std::cell::RefCell;
use std::rc::Rc;

static ACTION_DELAY: u32 = 5000;

#[derive(Clone, Debug)]
pub struct UndoBar {
    widget: InfoBar,
    label: Label,
    button: Button,
    current_action: GtkHandle<Option<UndoAction>>,
}

impl UndoBar {
    pub fn new(builder: &BuilderHelper) -> Self {
        let undo_bar = UndoBar {
            widget: builder.get::<InfoBar>("undo_bar"),
            label: builder.get::<Label>("undo_label"),
            button: builder.get::<Button>("undo_button"),
            current_action: gtk_handle!(None),
        };
        undo_bar.init();

        undo_bar
    }

    fn init(&self) {
        let button_info_bar = self.widget.clone();
        let button_current_action = self.current_action.clone();
        self.button.connect_clicked(move |_button| {
            if let Some(current_action) = button_current_action.borrow().as_ref() {
                GtkUtil::remove_source(Some(current_action.get_timeout()));
            }
            button_current_action.replace(None);
            button_info_bar.set_revealed(false);
        });
    }

    fn execute_action(action: &UndoActionType, bar: &InfoBar) {
        match action {
            UndoActionType::DeleteFeed((feed_id, _label)) => {
                if let Ok(main_window) = GtkUtil::get_main_window(bar) {
                    if let Some(action) = main_window.lookup_action("delete-feed") {
                        let variant = Variant::from(feed_id.to_str());
                        action.activate(Some(&variant));
                    }
                }
            }
            UndoActionType::DeleteCategory((category_id, _label)) => {
                if let Ok(main_window) = GtkUtil::get_main_window(bar) {
                    if let Some(action) = main_window.lookup_action("delete-category") {
                        let variant = Variant::from(category_id.to_str());
                        action.activate(Some(&variant));
                    }
                }
            }
        }
    }

    pub fn add_action(&self, action: UndoActionType) {
        if let Some(current_action) = self.current_action.borrow().as_ref() {
            GtkUtil::remove_source(Some(current_action.get_timeout()));
            Self::execute_action(current_action.get_type(), &self.widget);
        }

        match &action {
            UndoActionType::DeleteCategory((_id, label)) => {
                self.label.set_label(&format!("Delted Category'{}'", label))
            }
            UndoActionType::DeleteFeed((_id, label)) => self.label.set_label(&format!("Delted Feed'{}'", label)),
        }

        self.widget.set_revealed(true);

        let timeout_action = action.clone();
        let timeout_widget = self.widget.clone();
        let timeout_current_action = self.current_action.clone();
        let source_id = gtk::timeout_add(ACTION_DELAY, move || {
            Self::execute_action(&timeout_action, &timeout_widget);
            timeout_widget.set_revealed(false);
            timeout_current_action.replace(None);
            Continue(false)
        });

        self.current_action
            .borrow_mut()
            .replace(UndoAction::new(action, source_id.to_glib()));

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
