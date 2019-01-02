use gtk::{
    self,
};
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;

#[derive(Clone, Debug)]
pub struct WelcomePage {
    pub(crate) widget: gtk::Box,
}

impl WelcomePage {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/welcome_page.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("welcome_page").ok_or(format_err!("some err"))?;

        Ok(WelcomePage {
            widget: page.clone(),
        })
    }
}