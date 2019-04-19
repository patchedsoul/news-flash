use crate::Resources;
use failure::{Error};
use std::str;
use crate::util::{GTK_BUILDER_ERROR, GTK_RESOURCE_FILE_ERROR};

#[derive(Clone, Debug)]
pub struct WelcomeHeaderbar {
    widget: gtk::HeaderBar,
}

impl WelcomeHeaderbar {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/welcome_headerbar.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);

        let builder = gtk::Builder::new_from_string(ui_string);
        let headerbar: gtk::HeaderBar = builder.get_object("welcome_headerbar").expect(GTK_BUILDER_ERROR);

        Ok(WelcomeHeaderbar { widget: headerbar })
    }

    pub fn widget(&self) -> gtk::HeaderBar {
        self.widget.clone()
    }
}
