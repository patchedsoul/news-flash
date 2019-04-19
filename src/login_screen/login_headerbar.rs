use crate::Resources;
use gio::{ActionExt, ActionMapExt};
use crate::util::{GTK_RESOURCE_FILE_ERROR, GTK_BUILDER_ERROR};
use gtk::ButtonExt;
use std::str;

#[derive(Clone, Debug)]
pub struct LoginHeaderbar {
    widget: gtk::HeaderBar,
}

impl LoginHeaderbar {
    pub fn new(main_window: &gtk::ApplicationWindow) -> Self {
        let ui_data = Resources::get("ui/login_headerbar.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);

        let builder = gtk::Builder::new_from_string(ui_string);
        let headerbar: gtk::HeaderBar = builder.get_object("login_headerbar").expect(GTK_BUILDER_ERROR);
        let button: gtk::Button = builder.get_object("back_button").expect(GTK_BUILDER_ERROR);
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
