pub mod category_row;
pub mod feed_row;
pub mod models;
pub mod error;


use crate::Resources;
use crate::util::GtkUtil;
use std::str;
use std::collections::HashMap;
use failure::ResultExt;
use log::{
    debug,
};
use news_flash::models::{
    CategoryID,
    FeedID,
};
use crate::sidebar::feed_list::{
    category_row::CategoryRow,
    feed_row::FeedRow,
    models::{
        FeedListTree,
        FeedListCategoryModel,
        FeedListFeedModel,
        FeedListChangeSet,
        FeedListDndAction,
        FeedListSelection,
    },
};
use gtk::{
    self,
    ListBoxExt,
    ListBoxRowExt,
    StyleContextExt,
    ContainerExt,
    WidgetExt,
    WidgetExtManual,
    DestDefaults,
    TargetFlags,
    TargetEntry,
    Inhibit,
    SelectionMode,
};
use gdk::{
    EventType,
    DragAction,
};
use crate::sidebar::feed_list::error::{
    FeedListError,
    FeedListErrorKind,
};
use std::rc::Rc;
use std::cell::RefCell;
use crate::util::GtkHandle;
use crate::util::GtkHandleMap;
use crate::gtk_handle;


#[derive(Clone, Debug)]
pub struct FeedList {
    list: gtk::ListBox,
    categories: GtkHandleMap<CategoryID, GtkHandle<CategoryRow>>,
    feeds: GtkHandleMap<FeedID, GtkHandle<FeedRow>>,
    tree: GtkHandle<FeedListTree>,
}

impl FeedList {
    pub fn new() -> Result<Self, FeedListError> {
        let ui_data = Resources::get("ui/sidebar_list.ui").ok_or(FeedListErrorKind::UIFile)?;
        let ui_string = str::from_utf8(ui_data.as_ref()).context(FeedListErrorKind::EmbedFile)?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let list_box : gtk::ListBox = builder.get_object("sidebar_list").ok_or(FeedListErrorKind::UIFile)?;

        // set selection mode from NONE -> SINGLE after a delay after it's been shown
        // this ensures selection mode is in SINGLE without having a selected row in the list
        list_box.connect_show(|list| {
            let list = list.clone();
            gtk::timeout_add(50, move || {
                list.set_selection_mode(SelectionMode::Single);
                gtk::Continue(false)
            });
        });

        let feed_list = FeedList {
            list: list_box,
            categories: gtk_handle!(HashMap::new()),
            feeds: gtk_handle!(HashMap::new()),
            tree: gtk_handle!(FeedListTree::new()),
        };
        feed_list.setup_dnd();
        Ok(feed_list)
    }

    pub fn widget(&self) -> gtk::ListBox {
        self.list.clone()
    }

    fn setup_dnd(&self) {
        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        let tree = self.tree.clone();
        self.list.drag_dest_set(DestDefaults::DROP | DestDefaults::MOTION, &vec![entry], DragAction::MOVE);
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

                match y < alloc.y + (alloc.height / 2) {
                    true => {
                        if let Some(_) = widget.get_row_at_index(index - 1) {
                            if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                                ctx.add_class("drag-below");
                            }
                        }
                        else {
                            // row before doesn't exist -> insert at first pos
                            if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                                ctx.add_class("drag-below");
                            }
                        }
                    },
                    false => {
                        if let Some(row_below) = widget.get_row_at_index(index + 1) {
                            if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row_below) {
                                ctx.add_class("drag-below");
                            }
                        }
                        else {
                            // row after doesn't exist -> insert at last pos
                            if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                                ctx.add_class("drag-above");
                            }
                        }
                    },
                };
            }
            
            Inhibit(true)
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
        self.list.connect_drag_data_received(move |widget, _ctx, _x, y, selection_data, _info, _time| {
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

                let index = match y < alloc.y + (alloc.height / 2) {
                    true => {
                        match index - 1 >= 0 {
                            true => index - 1,
                            false => index,
                        }
                    },
                    false => {
                        match index + 1 >= 0 {
                            true => index + 1,
                            false => index,
                        }
                    },
                };

                if let Ok((parent_category, sort_index)) = tree.borrow().calculate_dnd(index).map_err(|_| {
                    debug!("Failed to calculate Drag&Drop action");
                }) {
                    if let Some(dnd_data_string) = selection_data.get_text() {
                        if dnd_data_string.contains("FeedID") {
                            let feed: FeedID = serde_json::from_str(&dnd_data_string.as_str().to_owned().split_off(6)).unwrap();
                            let _fixme = FeedListDndAction::MoveFeed(feed, parent_category.clone(), sort_index);
                        }

                        if dnd_data_string.contains("CategoryID") {
                            let category: CategoryID = serde_json::from_str(&dnd_data_string.as_str().to_owned().split_off(10)).unwrap();
                            let _fixme = FeedListDndAction::MoveCategory(category, parent_category.clone(), sort_index);
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
                        self.list.remove(&feed_handle.borrow().row());
                    }
                    self.feeds.borrow_mut().remove(&feed_id);
                },
                FeedListChangeSet::RemoveCategory(category_id) => {
                    if let Some(category_handle) = self.categories.borrow().get(&category_id) {
                        self.list.remove(&category_handle.borrow().row());
                    }
                    self.categories.borrow_mut().remove(&category_id);
                },
                FeedListChangeSet::AddFeed(model, pos, visible) => {
                    self.add_feed(&model, pos, visible);
                },
                FeedListChangeSet::AddCategory(model, pos, visible) => {
                    self.add_category(&model, pos, visible);
                },
                FeedListChangeSet::FeedUpdateItemCount(id, count) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&id) {
                        feed_handle.borrow().update_item_count(count);
                    }
                },
                FeedListChangeSet::CategoryUpdateItemCount(id, count) => {
                    if let Some(category_handle) = self.categories.borrow().get(&id) {
                        category_handle.borrow().update_item_count(count);
                    }
                },
                FeedListChangeSet::FeedUpdateLabel(id, label) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&id) {
                        feed_handle.borrow().update_title(&label);
                    }
                },
                FeedListChangeSet::CategoryUpdateLabel(id, label) => {
                    if let Some(category_handle) = self.categories.borrow().get(&id) {
                        category_handle.borrow().update_title(&label);
                    }
                },
            }
        }
    }

    fn add_category(&mut self, category: &FeedListCategoryModel, pos: i32, visible: bool) {
        let category_widget = CategoryRow::new(category, visible);
        let feeds = self.feeds.clone();
        let categories = self.categories.clone();
        let category_id = category.id.clone();
        let tree = self.tree.clone();
        self.list.insert(&category_widget.borrow().row(), pos);
        self.categories.borrow_mut().insert(category.id.clone(), category_widget.clone());

        category_widget.borrow().expander_event().connect_button_press_event(move |_widget, event| {
            if event.get_button() != 1 {
                return gtk::Inhibit(true)
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(true),
            }
            if let Some((feed_ids, category_ids, expaneded)) = tree.borrow_mut().collapse_expand_ids(&category_id) {
                for feed_id in feed_ids {
                    if let Some(feed_handle) = feeds.borrow().get(&feed_id) {
                        if expaneded {
                            feed_handle.borrow_mut().expand();
                        }
                        else {
                            feed_handle.borrow_mut().collapse();
                        }
                    }
                }
                for category_id in category_ids {
                    if let Some(category_handle) = categories.borrow().get(&category_id) {
                        if expaneded {
                            category_handle.borrow_mut().expand();
                        }
                        else {
                            category_handle.borrow_mut().collapse();
                        }
                    }
                }
            }
            gtk::Inhibit(true)
        });
    }

    fn add_feed(&mut self, feed: &FeedListFeedModel, pos: i32, visible: bool) {
        let feed_widget = FeedRow::new(feed, visible);
        self.list.insert(&feed_widget.borrow().row(), pos);
        self.feeds.borrow_mut().insert(feed.id.clone(), feed_widget);
    }

    pub fn deselect(&self) {
        self.list.unselect_all();
    }

    pub fn get_selection(&self) -> Option<FeedListSelection> {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.tree.borrow().calculate_selection(index)
        }
        None
    }
}