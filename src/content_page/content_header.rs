use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use gtk::{
    Builder,
};

pub struct ContentHeader {
    header: gtk::Paned,
}

impl ContentHeader {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/content_page_header.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let header : gtk::Paned = builder.get_object("content_header").ok_or(format_err!("some err"))?;

        Ok(ContentHeader {
            header: header,
        })
    }

    pub fn widget(&self) -> gtk::Paned {
        self.header.clone()
    }
}