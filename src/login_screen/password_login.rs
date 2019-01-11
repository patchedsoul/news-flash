use gtk::{
    self,
};
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use news_flash::NewsFlash;
use news_flash::models::{
    PluginMetadata,
};

#[derive(Clone, Debug)]
pub struct PasswordLogin {
    pub(crate) widget: gtk::Box,
}

impl PasswordLogin {
    pub fn new(info: PluginMetadata) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/password_login.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("password_login").ok_or(format_err!("some err"))?;

        let page = PasswordLogin {
            widget: page.clone(),
        };

        Ok(page)
    }
}