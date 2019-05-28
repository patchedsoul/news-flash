use crate::util::{BuilderHelper, GtkUtil, GTK_RESOURCE_FILE_ERROR};
use gio::{ActionExt, ActionMapExt};
use gtk::{Button, ButtonExt, HeaderBar};

#[derive(Clone, Debug)]
pub struct LoginHeaderbar {
    widget: gtk::HeaderBar,
}

impl LoginHeaderbar {
    pub fn new(builder: &BuilderHelper) -> Self {
        let headerbar = builder.get::<HeaderBar>("login_headerbar");
        let button = builder.get::<Button>("back_button");
        let main_window = GtkUtil::get_main_window(&headerbar).expect(GTK_RESOURCE_FILE_ERROR);
        button.connect_clicked(move |_button| {
            if let Some(action) = main_window.lookup_action("show-welcome-page") {
                action.activate(None);
            }
        });

        LoginHeaderbar { widget: headerbar }
    }
}
