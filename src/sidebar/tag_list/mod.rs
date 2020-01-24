mod error;
pub mod models;
mod tag_row;

use self::error::{TagListError, TagListErrorKind};
use crate::sidebar::SidebarIterateItem;
use crate::util::{BuilderHelper, GtkUtil};
use glib::translate::ToGlib;
use gtk::{ContainerExt, Continue, ListBox, ListBoxExt, ListBoxRowExt, SelectionMode, WidgetExt};
use models::{TagListChangeSet, TagListModel, TagListTagModel};
use news_flash::models::TagID;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tag_row::TagRow;

#[derive(Clone, Debug)]
pub struct TagList {
    list: ListBox,
    tags: HashMap<TagID, Arc<RwLock<TagRow>>>,
    list_model: Arc<RwLock<TagListModel>>,
    delayed_selection: Arc<RwLock<Option<u32>>>,
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
            list_model: Arc::new(RwLock::new(TagListModel::new())),
            delayed_selection: Arc::new(RwLock::new(None)),
        }
    }

    pub fn widget(&self) -> gtk::ListBox {
        self.list.clone()
    }

    pub fn update(&mut self, new_list: TagListModel) {
        let mut old_list = new_list;
        std::mem::swap(&mut old_list, &mut *self.list_model.write());

        let list_diff = old_list.generate_diff(&mut self.list_model.write());
        for diff in list_diff {
            match diff {
                TagListChangeSet::Remove(id) => {
                    if let Some(tag_handle) = self.tags.get(&id) {
                        self.list.remove(&tag_handle.read().widget());
                    }
                    self.tags.remove(&id);
                }
                TagListChangeSet::Add(model, pos) => {
                    self.add_tag(&model, pos);
                }
                TagListChangeSet::UpdateLabel(id, label) => {
                    if let Some(tag_handle) = self.tags.get(&id) {
                        tag_handle.read().update_title(&label);
                    }
                }
            }
        }
    }

    fn add_tag(&mut self, tag: &TagListTagModel, pos: i32) {
        let tag_widget = TagRow::new(tag);
        self.list.insert(&tag_widget.read().widget(), pos);
        self.tags.insert(tag.id.clone(), tag_widget);
    }

    pub fn deselect(&self) {
        self.list.unselect_all();
    }

    pub fn get_selection(&self) -> Option<(TagID, String)> {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            if let Some((_, model)) = self.list_model.read().calculate_selection(index) {
                return Some((model.id.clone(), model.label.clone()));
            }
        }
        None
    }

    pub fn get_next_item(&self) -> SidebarIterateItem {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.list_model.read().calculate_next_item(index);
        }
        SidebarIterateItem::NothingSelected
    }

    pub fn get_prev_item(&self) -> SidebarIterateItem {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.list_model.read().calculate_prev_item(index);
        }
        SidebarIterateItem::NothingSelected
    }

    pub fn get_first_item(&self) -> Option<TagID> {
        self.list_model.write().first().map(|model| model.id.clone())
    }

    pub fn get_last_item(&self) -> Option<TagID> {
        self.list_model.write().last().map(|model| model.id.clone())
    }

    pub fn set_selection(&self, selection: TagID) -> Result<(), TagListError> {
        self.cancel_selection();

        if let Some(tag_row) = self.tags.get(&selection) {
            let list = self.list.clone();
            let delayed_selection = self.delayed_selection.clone();
            let row = tag_row.read().widget();
            gtk::idle_add(move || {
                list.select_row(Some(&row));

                let active_row = row.clone();
                let source_id = delayed_selection.clone();
                *delayed_selection.write() = Some(
                    gtk::timeout_add(300, move || {
                        active_row.emit_activate();
                        *source_id.write() = None;
                        Continue(false)
                    })
                    .to_glib(),
                );

                row.emit_activate();
                Continue(false)
            });
            return Ok(());
        }

        Err(TagListErrorKind::InvalidSelection.into())
    }

    pub fn cancel_selection(&self) {
        GtkUtil::remove_source(*self.delayed_selection.read());
        *self.delayed_selection.write() = None;
    }
}
