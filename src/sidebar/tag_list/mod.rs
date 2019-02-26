mod models;

use news_flash::models::{
    TagID,
    Tag,
};
use failure::{
    Error,
    ResultExt,
    format_err,
};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str;
use crate::Resources;
use crate::main_window::GtkHandle;
use crate::main_window::GtkHandleMap;

#[derive(Clone, Debug)]
pub struct TagList {
    list: gtk::ListBox,
    //tags: GtkHandleMap<TagID, GtkHandle<TagRow>>,
    //list_model: GtkHandle<TagListModel>,
}

impl TagList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/sidebar_list.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref()).context(format_err!("some err"))?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let list_box : gtk::ListBox = builder.get_object("sidebar_list").ok_or(format_err!("some err"))?;

        let tag_list = TagList {
            list: list_box,
            //tags: Rc::new(RefCell::new(HashMap::new())),
            //list_model: Rc::new(RefCell::new(/* FIXME */)),
        };
        Ok(tag_list)
    }

    pub fn widget(&self) -> gtk::ListBox {
        self.list.clone()
    }
}