use gtk::{
    self,
    ListBoxExt,
};
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use news_flash::NewsFlash;
use super::service_row::ServiceRow;

#[derive(Clone, Debug)]
pub struct WelcomePage {
    page: gtk::Box,
    list: gtk::ListBox,
}

impl WelcomePage {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/welcome_page.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("welcome_page").ok_or(format_err!("some err"))?;
        let list : gtk::ListBox = builder.get_object("list").ok_or(format_err!("some err"))?;

        let page = WelcomePage {
            page: page,
            list: list,
        };

        page.populate()?;

        Ok(page)
    }

    fn populate(&self) -> Result<(), Error> {
        let services = NewsFlash::list_backends();
        for (_id, api_meta) in services {
            let service_meta = api_meta.metadata();
            let row = ServiceRow::new(service_meta)?;
            self.list.insert(&row.widget(), -1);
        }
        Ok(())
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }
}