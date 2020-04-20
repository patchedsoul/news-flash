mod models;

use crate::app::Action;
use crate::i18n::i18n_f;
use crate::util::{BuilderHelper, GtkUtil, Util};
use glib::{clone, source::Continue, translate::ToGlib, Sender};
use gtk::{Button, ButtonExt, InfoBar, InfoBarExt, Label, LabelExt, ResponseType, WidgetExt};
use log::debug;
pub use models::{UndoAction, UndoActionModel};
use parking_lot::RwLock;
use std::sync::Arc;

static ACTION_DELAY: u32 = 10000;

#[derive(Clone, Debug)]
pub struct UndoBar {
    widget: InfoBar,
    label: Label,
    button: Button,
    current_action: Arc<RwLock<Option<UndoAction>>>,
    sender: Sender<Action>,
}

impl UndoBar {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let undo_bar = UndoBar {
            widget: builder.get::<InfoBar>("undo_bar"),
            label: builder.get::<Label>("undo_label"),
            button: builder.get::<Button>("undo_button"),
            current_action: Arc::new(RwLock::new(None)),
            sender,
        };
        undo_bar.init();

        undo_bar
    }

    fn init(&self) {
        self.button.connect_clicked(clone!(
            @weak self.widget as info_bar,
            @weak self.current_action as current_action,
            @strong self.sender as sender => @default-panic, move |_button|
        {
            if let Some(current_action) = current_action.read().as_ref() {
                GtkUtil::remove_source(Some(current_action.get_timeout()));
            }
            current_action.write().take();
            info_bar.set_revealed(false);

            // update lists
            Util::send(&sender, Action::UpdateSidebar);
            Util::send(&sender, Action::UpdateArticleList);
        }));

        self.widget.connect_response(clone!(
            @weak self.current_action as current_action,
            @strong self.sender as sender => @default-panic, move |info_bar, response|
        {
            if response == ResponseType::Close {
                if let Some(current_action) = current_action.read().as_ref() {
                    Self::execute_action(&current_action.get_model(), &sender);
                    GtkUtil::remove_source(Some(current_action.get_timeout()));
                }

                current_action.write().take();
                info_bar.set_revealed(false);
            }
        }));

        self.widget.show();
    }

    fn execute_action(action: &UndoActionModel, sender: &Sender<Action>) {
        let sender = sender.clone();
        match action {
            UndoActionModel::DeleteFeed((feed_id, _label)) => {
                Util::send(&sender, Action::DeleteFeed(feed_id.clone()));
            }
            UndoActionModel::DeleteCategory((category_id, _label)) => {
                Util::send(&sender, Action::DeleteCategory(category_id.clone()));
            }
            UndoActionModel::DeleteTag((tag_id, _label)) => {
                Util::send(&sender, Action::DeleteTag(tag_id.clone()));
            }
        }
    }

    pub fn add_action(&self, action: UndoActionModel) {
        if let Some(current_action) = self.current_action.read().as_ref() {
            debug!("remove current action: {}", current_action.get_model());
            GtkUtil::remove_source(Some(current_action.get_timeout()));
            Self::execute_action(current_action.get_model(), &self.sender);
        }

        match &action {
            UndoActionModel::DeleteCategory((_id, label)) => {
                self.label.set_label(&i18n_f("Deleted Category '{}'", &[label]))
            }
            UndoActionModel::DeleteFeed((_id, label)) => self.label.set_label(&i18n_f("Deleted Feed '{}'", &[label])),
            UndoActionModel::DeleteTag((_id, label)) => self.label.set_label(&i18n_f("Deleted Tag '{}'", &[label])),
        }

        self.widget.set_revealed(true);

        let source_id = gtk::timeout_add(
            ACTION_DELAY,
            clone!(
                @strong action,
                @weak self.widget as widget,
                @weak self.current_action as current_action,
                @strong self.sender as sender => @default-panic, move ||
            {
                Self::execute_action(&action, &sender);
                widget.set_revealed(false);
                current_action.write().take();
                Continue(false)
            }),
        );

        self.current_action
            .write()
            .replace(UndoAction::new(action, source_id.to_glib()));

        // update lists
        Util::send(&self.sender, Action::UpdateSidebar);
        Util::send(&self.sender, Action::UpdateArticleList);
    }

    pub fn get_current_action(&self) -> Option<UndoActionModel> {
        if let Some(current_action) = self.current_action.read().as_ref() {
            return Some(current_action.get_model().clone());
        }
        None
    }

    pub fn execute_pending_action(&self) {
        if let Some(current_action) = self.get_current_action() {
            Self::execute_action(&current_action, &self.sender);
        }
    }
}
