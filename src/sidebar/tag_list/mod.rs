pub mod models;
mod tag_row;

use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle};
use gtk::{ContainerExt, ListBox, ListBoxExt, ListBoxRowExt, SelectionMode, WidgetExt};
use models::{TagListChangeSet, TagListModel, TagListTagModel};
use news_flash::models::TagID;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tag_row::TagRow;

#[derive(Clone, Debug)]
pub struct TagList {
    list: ListBox,
    tags: HashMap<TagID, GtkHandle<TagRow>>,
    list_model: GtkHandle<TagListModel>,
}

impl TagList {
    pub fn new() -> Self {
        let builder = BuilderHelper::new("sidebar_list");
        let list_box = builder.get::<ListBox>("sidebar_list");

        // set selection mode from NONE -> SINGLE after a delay after it's been shown
        // this ensures selection mode is in SINGLE without having a selected row in the list
        list_box.connect_show(|list| {
            let list = list.clone();
            gtk::timeout_add(50, move || {
                list.set_selection_mode(SelectionMode::Single);
                gtk::Continue(false)
            });
        });

        TagList {
            list: list_box,
            tags: HashMap::new(),
            list_model: gtk_handle!(TagListModel::new()),
        }
    }

    pub fn widget(&self) -> gtk::ListBox {
        self.list.clone()
    }

    pub fn update(&mut self, new_list: TagListModel) {
        let old_list = self.list_model.clone();
        self.list_model = gtk_handle!(new_list);
        let list_diff = old_list.borrow_mut().generate_diff(&mut self.list_model.borrow_mut());
        for diff in list_diff {
            match diff {
                TagListChangeSet::Remove(id) => {
                    if let Some(tag_handle) = self.tags.get(&id) {
                        self.list.remove(&tag_handle.borrow().row());
                    }
                    self.tags.remove(&id);
                }
                TagListChangeSet::Add(model, pos) => {
                    self.add_tag(&model, pos);
                }
                TagListChangeSet::UpdateItemCount(id, count) => {
                    if let Some(tag_handle) = self.tags.get(&id) {
                        tag_handle.borrow().update_item_count(count);
                    }
                }
                TagListChangeSet::UpdateLabel(id, label) => {
                    if let Some(tag_handle) = self.tags.get(&id) {
                        tag_handle.borrow().update_title(&label);
                    }
                }
            }
        }
    }

    fn add_tag(&mut self, tag: &TagListTagModel, pos: i32) {
        let tag_widget = TagRow::new(tag);
        self.list.insert(&tag_widget.borrow().row(), pos);
        self.tags.insert(tag.id.clone(), tag_widget);
    }

    pub fn deselect(&self) {
        self.list.unselect_all();
    }

    pub fn get_selection(&self) -> Option<TagID> {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            if let Some((_, model)) = self.list_model.borrow().calculate_selection(index) {
                return Some(model.id.clone());
            }
        }
        None
    }
}
