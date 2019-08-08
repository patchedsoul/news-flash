use crate::undo_bar::UndoActionModel;
use crate::util::{BuilderHelper, GtkUtil};
use gio::{ActionExt, ActionMapExt};
use glib::Variant;
use gtk::{Button, ButtonExt};
use news_flash::models::FeedID;

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
                if let Ok(selection_json) =
                    serde_json::to_string(&UndoActionModel::DeleteFeed((FeedID::new("asdf"), "asdfa".to_owned())))
                {
                    if let Some(action) = main_window.lookup_action("enqueue-undoable-action") {
                        let selection = Variant::from(&selection_json);
                        action.activate(Some(&selection));
                    }
                }
            }
        });

        SidebarFooter {
            add_button,
            remove_button,
        }
    }
}
