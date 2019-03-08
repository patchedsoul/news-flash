use gtk::{
    Builder,
};
use std::str;
use failure::Error;
use failure::format_err;
use crate::Resources;

pub struct SingleArticleList {
    scroll: gtk::ScrolledWindow,
    list: gtk::ListBox,
}

impl SingleArticleList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_list_single.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let scroll : gtk::ScrolledWindow = builder.get_object("article_list_scroll").ok_or(format_err!("some err"))?;
        let list : gtk::ListBox = builder.get_object("article_list_box").ok_or(format_err!("some err"))?;

        Ok(SingleArticleList {
            scroll: scroll,
            list: list,
        })
    }

    pub fn widget(&self) -> gtk::ScrolledWindow {
        self.scroll.clone()
    }
}