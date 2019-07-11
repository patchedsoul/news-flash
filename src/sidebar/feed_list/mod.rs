pub mod category_row;
pub mod error;
pub mod feed_row;
pub mod models;

use crate::gtk_handle;
use crate::sidebar::feed_list::error::{FeedListError, FeedListErrorKind};
use crate::sidebar::feed_list::{
    category_row::CategoryRow,
    feed_row::FeedRow,
    models::{
        FeedListCategoryModel, FeedListChangeSet, FeedListDndAction, FeedListFeedModel, FeedListItem, FeedListItemID,
        FeedListTree,
    },
};
use crate::sidebar::SidebarIterateItem;
use crate::util::{BuilderHelper, GtkHandle, GtkHandleMap, GtkUtil};
use gdk::{DragAction, EventType};
use glib::translate::ToGlib;
use gtk::{
    self, ContainerExt, Continue, DestDefaults, Inhibit, ListBox, ListBoxExt, ListBoxRowExt, SelectionMode,
    StyleContextExt, TargetEntry, TargetFlags, WidgetExt, WidgetExtManual,
};
use log::debug;
use news_flash::models::{CategoryID, FeedID};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct FeedList {
    list: ListBox,
    categories: GtkHandleMap<CategoryID, GtkHandle<CategoryRow>>,
    feeds: GtkHandleMap<FeedID, GtkHandle<FeedRow>>,
    tree: GtkHandle<FeedListTree>,
    delayed_selection: GtkHandle<Option<u32>>,
}

impl FeedList {
    pub fn new() -> Self {
        let builder = BuilderHelper::new("sidebar_list");
        let list_box = builder.get::<ListBox>("sidebar_list");

        // set selection mode from NONE -> SINGLE after a delay after it's been shown
        // this ensures selection mode is in SINGLE without having a selected row in the list
        list_box.connect_show(|list| {
            let list = list.clone();
            gtk::timeout_add(50, move || {
                list.set_selection_mode(SelectionMode::Single);
                Continue(false)
            });
        });

        let feed_list = FeedList {
            list: list_box,
            categories: gtk_handle!(HashMap::new()),
            feeds: gtk_handle!(HashMap::new()),
            tree: gtk_handle!(FeedListTree::new()),
            delayed_selection: gtk_handle!(None),
        };
        feed_list.setup_dnd();
        feed_list
    }

    pub fn widget(&self) -> ListBox {
        self.list.clone()
    }

    fn setup_dnd(&self) {
        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        let tree = self.tree.clone();
        self.list
            .drag_dest_set(DestDefaults::DROP | DestDefaults::MOTION, &[entry], DragAction::MOVE);
        self.list.drag_dest_add_text_targets();
        self.list.connect_drag_motion(|widget, _drag_context, _x, y, _time| {
            // maybe we should keep track of the previous highlighted rows instead of iterating over all of them
            let children = widget.get_children();
            for widget in children {
                if let Some(ctx) = GtkUtil::get_dnd_style_context_widget(&widget) {
                    ctx.remove_class("drag-above");
                    ctx.remove_class("drag-below");
                }
            }

            if let Some(row) = widget.get_row_at_y(y) {
                let alloc = row.get_allocation();
                let index = row.get_index();

                if y < alloc.y + (alloc.height / 2) {
                    if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                        ctx.add_class("drag-below");
                    }
                } else if let Some(row_below) = widget.get_row_at_index(index + 1) {
                    if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row_below) {
                        ctx.add_class("drag-below");
                    }
                } else {
                    // row after doesn't exist -> insert at last pos
                    if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                        ctx.add_class("drag-above");
                    }
                }
            }

            Inhibit(false)
        });
        self.list.connect_drag_leave(|widget, _drag_context, _time| {
            let children = widget.get_children();
            for widget in children {
                if let Some(ctx) = GtkUtil::get_dnd_style_context_widget(&widget) {
                    ctx.remove_class("drag-above");
                    ctx.remove_class("drag-below");
                }
            }
        });
        self.list
            .connect_drag_data_received(move |widget, _ctx, _x, y, selection_data, _info, _time| {
                let children = widget.get_children();
                for widget in children {
                    if let Some(ctx) = GtkUtil::get_dnd_style_context_widget(&widget) {
                        ctx.remove_class("drag-above");
                        ctx.remove_class("drag-below");
                    }
                }

                if let Some(row) = widget.get_row_at_y(y) {
                    let alloc = row.get_allocation();
                    let index = row.get_index();

                    let index = if y < alloc.y + (alloc.height / 2) {
                        if index > 0 {
                            index - 1
                        } else {
                            index
                        }
                    } else if index + 1 >= 0 {
                        index + 1
                    } else {
                        index
                    };

                    if let Ok((parent_category, sort_index)) = tree.borrow().calculate_dnd(index).map_err(|_| {
                        debug!("Failed to calculate Drag&Drop action");
                    }) {
                        if let Some(dnd_data_string) = selection_data.get_text() {
                            if dnd_data_string.contains("FeedID") {
                                let feed: FeedID =
                                    serde_json::from_str(&dnd_data_string.as_str().to_owned().split_off(6)).unwrap();
                                let _fixme = FeedListDndAction::MoveFeed(feed, parent_category.clone(), sort_index);
                            }

                            if dnd_data_string.contains("CategoryID") {
                                let category: CategoryID =
                                    serde_json::from_str(&dnd_data_string.as_str().to_owned().split_off(10)).unwrap();
                                let _fixme =
                                    FeedListDndAction::MoveCategory(category, parent_category.clone(), sort_index);
                            }
                        }
                    }
                }
            });
    }

    pub fn update(&mut self, new_tree: FeedListTree) {
        let old_tree = self.tree.clone();
        self.tree = gtk_handle!(new_tree);
        let tree_diff = old_tree.borrow().generate_diff(&self.tree.borrow());
        for diff in tree_diff {
            match diff {
                FeedListChangeSet::RemoveFeed(feed_id) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&feed_id) {
                        self.list.remove(&feed_handle.borrow().widget());
                    }
                    self.feeds.borrow_mut().remove(&feed_id);
                }
                FeedListChangeSet::RemoveCategory(category_id) => {
                    if let Some(category_handle) = self.categories.borrow().get(&category_id) {
                        self.list.remove(&category_handle.borrow().widget());
                    }
                    self.categories.borrow_mut().remove(&category_id);
                }
                FeedListChangeSet::AddFeed(model, pos, visible) => {
                    self.add_feed(&model, pos, visible);
                }
                FeedListChangeSet::AddCategory(model, pos, visible) => {
                    self.add_category(&model, pos, visible);
                }
                FeedListChangeSet::FeedUpdateItemCount(id, count) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&id) {
                        feed_handle.borrow().update_item_count(count);
                    }
                }
                FeedListChangeSet::CategoryUpdateItemCount(id, count) => {
                    if let Some(category_handle) = self.categories.borrow().get(&id) {
                        category_handle.borrow().update_item_count(count);
                    }
                }
                FeedListChangeSet::FeedUpdateLabel(id, label) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&id) {
                        feed_handle.borrow().update_title(&label);
                    }
                }
                FeedListChangeSet::CategoryUpdateLabel(id, label) => {
                    if let Some(category_handle) = self.categories.borrow().get(&id) {
                        category_handle.borrow().update_title(&label);
                    }
                }
            }
        }
    }

    fn add_category(&mut self, category: &FeedListCategoryModel, pos: i32, visible: bool) {
        let category_widget = CategoryRow::new(category, visible);
        let feeds = self.feeds.clone();
        let categories = self.categories.clone();
        let category_id = category.id.clone();
        let tree = self.tree.clone();
        self.list.insert(&category_widget.borrow().widget(), pos);
        self.categories
            .borrow_mut()
            .insert(category.id.clone(), category_widget.clone());

        category_widget
            .borrow()
            .expander_event()
            .connect_button_press_event(move |_widget, event| {
                if event.get_button() != 1 {
                    return Inhibit(false);
                }
                match event.get_event_type() {
                    EventType::ButtonPress => (),
                    _ => return Inhibit(false),
                }
                Self::expand_collapse_category(&category_id, &tree, &categories, &feeds);
                Inhibit(true)
            });
    }

    pub fn expand_collapse_selected_category(&self) {
        if let Some(row) = self.list.get_selected_row() {
            let selection = self.tree.borrow().calculate_selection(row.get_index());
            if let Some(selected_item) = selection {
                if let (FeedListItemID::Category(id), _title) = selected_item {
                    Self::expand_collapse_category(&id, &self.tree, &self.categories, &self.feeds);
                }
            }
        }
    }

    fn expand_collapse_category(
        category_id: &CategoryID,
        tree: &GtkHandle<FeedListTree>,
        categories: &GtkHandleMap<CategoryID, GtkHandle<CategoryRow>>,
        feeds: &GtkHandleMap<FeedID, GtkHandle<FeedRow>>,
    ) {
        if let Some((feed_ids, category_ids, expaneded)) = tree.borrow_mut().collapse_expand_ids(category_id) {
            for feed_id in feed_ids {
                if let Some(feed_handle) = feeds.borrow().get(&feed_id) {
                    if expaneded {
                        feed_handle.borrow_mut().expand();
                    } else {
                        feed_handle.borrow_mut().collapse();
                    }
                }
            }
            for category_id in category_ids {
                if let Some(category_handle) = categories.borrow().get(&category_id) {
                    if expaneded {
                        category_handle.borrow_mut().expand();
                    } else {
                        category_handle.borrow_mut().collapse();
                    }
                }
            }
        }
    }

    fn add_feed(&mut self, feed: &FeedListFeedModel, pos: i32, visible: bool) {
        let feed_widget = FeedRow::new(feed, visible);
        self.list.insert(&feed_widget.borrow().widget(), pos);
        self.feeds.borrow_mut().insert(feed.id.clone(), feed_widget);
    }

    pub fn deselect(&self) {
        self.list.unselect_all();
    }

    pub fn get_selection(&self) -> Option<(FeedListItemID, String)> {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.tree.borrow().calculate_selection(index);
        }
        None
    }

    pub fn get_first_item(&self) -> Option<FeedListItemID> {
        self.tree.borrow().top_level.first().map(|item| match item {
            FeedListItem::Feed(item) => FeedListItemID::Feed(item.id.clone()),
            FeedListItem::Category(item) => FeedListItemID::Category(item.id.clone()),
        })
    }

    pub fn get_last_item(&self, last_item: Option<FeedListItem>) -> Option<FeedListItemID> {
        let last_item = if last_item.is_some() {
            last_item
        } else {
            self.tree.borrow().top_level.last().cloned()
        };

        if let Some(last) = last_item {
            match last {
                FeedListItem::Feed(item) => return Some(FeedListItemID::Feed(item.id.clone())),
                FeedListItem::Category(item) => {
                    if item.expanded {
                        if item.children.is_empty() {
                            return Some(FeedListItemID::Category(item.id.clone()));
                        } else {
                            return self.get_last_item(item.children.last().cloned());
                        }
                    } else {
                        return Some(FeedListItemID::Category(item.id.clone()));
                    }
                }
            }
        }
        None
    }

    pub fn set_selection(&self, selection: FeedListItemID) -> Result<(), FeedListError> {
        self.cancel_selection();

        let row = match selection {
            FeedListItemID::Category(category) => match self.categories.borrow().get(&category) {
                Some(category_row) => category_row.borrow().widget(),
                None => return Err(FeedListErrorKind::CategoryNotFound)?,
            },
            FeedListItemID::Feed(feed) => match self.feeds.borrow().get(&feed) {
                Some(feed_row) => feed_row.borrow().widget(),
                None => return Err(FeedListErrorKind::FeedNotFound)?,
            },
        };

        let list = self.list.clone();
        let selected_row = row.clone();
        let delayed_selection = self.delayed_selection.clone();
        gtk::idle_add(move || {
            list.select_row(Some(&selected_row));

            let row = row.clone();
            let source_id = delayed_selection.clone();
            *delayed_selection.borrow_mut() = Some(
                gtk::timeout_add(300, move || {
                    row.emit_activate();
                    *source_id.borrow_mut() = None;
                    Continue(false)
                })
                .to_glib(),
            );

            Continue(false)
        });

        Ok(())
    }

    pub fn cancel_selection(&self) {
        GtkUtil::remove_source(*self.delayed_selection.borrow());
        *self.delayed_selection.borrow_mut() = None;
    }

    pub fn select_next_item(&self) -> SidebarIterateItem {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.tree.borrow_mut().calculate_next_item(index);
        }
        SidebarIterateItem::NothingSelected
    }

    pub fn select_prev_item(&self) -> SidebarIterateItem {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.tree.borrow_mut().calculate_prev_item(index);
        }
        SidebarIterateItem::NothingSelected
    }
}
