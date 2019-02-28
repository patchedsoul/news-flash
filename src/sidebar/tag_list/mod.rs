pub mod models;
mod tag_row;

use news_flash::models::{
    TagID,
};
use failure::{
    Error,
    ResultExt,
    format_err,
};
use gtk::{
    ListBoxExt,
    ContainerExt,
};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str;
use crate::Resources;
use crate::main_window::GtkHandle;
use crate::main_window::GtkHandleMap;
use models::{
    TagListModel,
    TagListChangeSet,
    TagListTagModel,
};
use tag_row::TagRow;

#[derive(Clone, Debug)]
pub struct TagList {
    list: gtk::ListBox,
    tags: GtkHandleMap<TagID, GtkHandle<TagRow>>,
    list_model: GtkHandle<TagListModel>,
}

impl TagList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/sidebar_list.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref()).context(format_err!("some err"))?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let list_box : gtk::ListBox = builder.get_object("sidebar_list").ok_or(format_err!("some err"))?;

        let tag_list = TagList {
            list: list_box,
            tags: Rc::new(RefCell::new(HashMap::new())),
            list_model: Rc::new(RefCell::new(TagListModel::new())),
        };
        Ok(tag_list)
    }

    pub fn widget(&self) -> gtk::ListBox {
        self.list.clone()
    }

    pub fn update(&mut self, new_list: TagListModel) {
        let old_list = self.list_model.clone();
        self.list_model = Rc::new(RefCell::new(new_list));
        let list_diff = old_list.borrow().generate_diff(&self.list_model.borrow());
        for diff in list_diff {
            match diff {
                TagListChangeSet::Remove(id) => {
                    if let Some(tag_handle) = self.tags.borrow().get(&id) {
                        self.list.remove(&tag_handle.borrow().row());
                    }
                    self.tags.borrow_mut().remove(&id);
                },
                TagListChangeSet::Add(model, pos) => {
                    self.add_tag(&model, pos);
                },
                TagListChangeSet::UpdateItemCount(id, count) => {
                    if let Some(tag_handle) = self.tags.borrow().get(&id) {
                        tag_handle.borrow().update_item_count(count);
                    }
                },
                TagListChangeSet::UpdateLabel(id, label) => {
                    if let Some(tag_handle) = self.tags.borrow().get(&id) {
                        tag_handle.borrow().update_title(&label);
                    }
                },
            }
        }
    }

    fn add_tag(&mut self, tag: &TagListTagModel, pos: i32) {
        let tag_widget = TagRow::new(tag);
        self.list.insert(&tag_widget.borrow().row(), pos);
        self.tags.borrow_mut().insert(tag.id.clone(), tag_widget);
    }
}