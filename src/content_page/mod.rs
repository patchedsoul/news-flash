use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use gtk::{
    Builder,
    BoxExt,
};
use crate::sidebar::SideBar;
use news_flash::models::{
    PluginID,
};

pub struct ContentPage {
    page: gtk::Box,
    sidebar: SideBar,
}

impl ContentPage {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/content_page.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("page").ok_or(format_err!("some err"))?;
        let feed_list_box : gtk::Box = builder.get_object("feedlist_box").ok_or(format_err!("some err"))?;
        
        let sidebar = SideBar::new()?;

        feed_list_box.pack_start(&sidebar.widget(), false, true, 0);

        Ok(ContentPage {
            page: page,
            sidebar: sidebar,
        })
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }

    pub fn set_service(&self, id: &PluginID) -> Result<(), Error> {
        self.sidebar.set_service(id)?;
        Ok(())
    }
}