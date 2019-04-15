use crate::Resources;
use failure::{format_err, Error};
use std::str;

#[derive(Clone, Debug)]
pub struct WelcomeHeaderbar {
    widget: gtk::HeaderBar,
}

impl WelcomeHeaderbar {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/welcome_headerbar.ui").ok_or_else(|| format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let headerbar: gtk::HeaderBar = builder.get_object("welcome_headerbar").ok_or_else(|| format_err!("some err"))?;

        Ok(WelcomeHeaderbar { widget: headerbar })
    }

    pub fn widget(&self) -> gtk::HeaderBar {
        self.widget.clone()
    }
}
