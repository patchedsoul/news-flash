mod models;

use crate::app::Action;
use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil, Util};
use glib::{translate::ToGlib, Sender, Variant};
use gtk::{Button, ButtonExt, Continue, InfoBar, InfoBarExt, Label, LabelExt, WidgetExt};
use log::debug;
pub use models::{UndoAction, UndoActionModel};
use std::cell::RefCell;
use std::rc::Rc;

static ACTION_DELAY: u32 = 10000;

#[derive(Clone, Debug)]
pub struct UndoBar {
    widget: InfoBar,
    label: Label,
    button: Button,
    current_action: GtkHandle<Option<UndoAction>>,
    sender: Sender<Action>,
}

impl UndoBar {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let undo_bar = UndoBar {
            widget: builder.get::<InfoBar>("undo_bar"),
            label: builder.get::<Label>("undo_label"),
            button: builder.get::<Button>("undo_button"),
            current_action: gtk_handle!(None),
            sender,
        };
        undo_bar.init();

        undo_bar
    }

    fn init(&self) {
        let button_info_bar = self.widget.clone();
        let button_current_action = self.current_action.clone();
        let sender = self.sender.clone();
        self.button.connect_clicked(move |_button| {
            if let Some(current_action) = button_current_action.borrow().as_ref() {
                GtkUtil::remove_source(Some(current_action.get_timeout()));
            }
            button_current_action.replace(None);
            button_info_bar.set_revealed(false);

            // update lists
            Util::send(&sender, Action::UpdateSidebar);
            Util::send(&sender, Action::UpdateArticleList);
        });

        self.widget.show();
    }

    fn execute_action(action: &UndoActionModel, info_bar: &InfoBar) {
        match action {
            UndoActionModel::DeleteFeed((feed_id, _label)) => {
                let variant = Variant::from(feed_id.to_str());
                GtkUtil::execute_action(info_bar, "delete-feed", Some(&variant));
            }
            UndoActionModel::DeleteCategory((category_id, _label)) => {
                let variant = Variant::from(category_id.to_str());
                GtkUtil::execute_action(info_bar, "delete-category", Some(&variant));
            }
            UndoActionModel::DeleteTag((tag_id, _label)) => {
                let variant = Variant::from(tag_id.to_str());
                GtkUtil::execute_action(info_bar, "delete-tag", Some(&variant));
            }
        }
    }

    pub fn add_action(&self, action: UndoActionModel) {
        if let Some(current_action) = self.current_action.borrow().as_ref() {
            debug!("remove current action: {}", current_action.get_model());
            GtkUtil::remove_source(Some(current_action.get_timeout()));
            Self::execute_action(current_action.get_model(), &self.widget);
        }

        match &action {
            UndoActionModel::DeleteCategory((_id, label)) => {
                self.label.set_label(&format!("Deleted Category '{}'", label))
            }
            UndoActionModel::DeleteFeed((_id, label)) => self.label.set_label(&format!("Deleted Feed '{}'", label)),
            UndoActionModel::DeleteTag((_id, label)) => self.label.set_label(&format!("Deleted Tag '{}'", label)),
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
        Util::send(&self.sender, Action::UpdateSidebar);
        Util::send(&self.sender, Action::UpdateArticleList);
    }

    pub fn get_current_action(&self) -> Option<UndoActionModel> {
        if let Some(current_action) = self.current_action.borrow().as_ref() {
            return Some(current_action.get_model().clone());
        }
        None
    }
}
