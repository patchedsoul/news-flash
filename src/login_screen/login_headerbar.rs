use gio::{ActionExt, ActionMapExt};
use gtk::{Button, ButtonExt, HeaderBar};
use crate::util::BuilderHelper;

#[derive(Clone, Debug)]
pub struct LoginHeaderbar {
    widget: gtk::HeaderBar,
}

impl LoginHeaderbar {
    pub fn new(main_window: &gtk::ApplicationWindow) -> Self {
        let builder = BuilderHelper::new("login_headerbar");
        let headerbar = builder.get::<HeaderBar>("login_headerbar");
        let button = builder.get::<Button>("back_button");
        let main_window = main_window.clone();
        button.connect_clicked(move |_button| {
            if let Some(action) = main_window.lookup_action("show-welcome-page") {
                action.activate(None);
            }
        });

        LoginHeaderbar { widget: headerbar }
    }

    pub fn widget(&self) -> gtk::HeaderBar {
        self.widget.clone()
    }
}
