//mod feed_list;

//pub use crate::sidebar::feed_list::category_row::CategoryRow;
//pub use crate::sidebar::feed_list::feed_row::FeedRow;
//pub use crate::sidebar::feed_list::feed_list::FeedList;

use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use gtk::{
    Builder,
};

pub struct SideBar {
    sidebar: gtk::Box,
}

impl SideBar {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/sidebar.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let sidebar : gtk::Box = builder.get_object("toplevel").ok_or(format_err!("some err"))?;

        Ok(SideBar {
            sidebar: sidebar,
        })
    }

    pub fn widget(&self) -> gtk::Box {
        self.sidebar.clone()
    }
}