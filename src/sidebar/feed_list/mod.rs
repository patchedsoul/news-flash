pub mod category_row;
pub mod error;
pub mod feed_row;
pub mod models;

use crate::app::Action;
use crate::main_window_state::MainWindowState;
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
use crate::util::{BuilderHelper, GtkUtil, Util};
use gdk::{DragAction, EventType};
use glib::{clone, source::Continue, translate::ToGlib, Sender};
use gtk::{
    self, prelude::WidgetExtManual, ContainerExt, DestDefaults, Inhibit, ListBox, ListBoxExt, ListBoxRowExt,
    ScrolledWindow, SelectionMode, StyleContextExt, TargetEntry, TargetFlags, WidgetExt,
};
use log::error;
use news_flash::models::{CategoryID, FeedID, PluginCapabilities};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct FeedList {
    list: ListBox,
    scroll: ScrolledWindow,
    sender: Sender<Action>,
    categories: Arc<RwLock<HashMap<CategoryID, Arc<RwLock<CategoryRow>>>>>,
    feeds: Arc<RwLock<HashMap<FeedID, Arc<RwLock<FeedRow>>>>>,
    tree: Arc<RwLock<FeedListTree>>,
    delayed_selection: Arc<RwLock<Option<u32>>>,
    hovered_category_expand: Arc<RwLock<Option<(u32, CategoryID)>>>,
    state: Arc<RwLock<MainWindowState>>,
}

impl FeedList {
    pub fn new(sidebar_scroll: &ScrolledWindow, state: &Arc<RwLock<MainWindowState>>, sender: Sender<Action>) -> Self {
        let builder = BuilderHelper::new("sidebar_list");
        let list_box = builder.get::<ListBox>("sidebar_list");

        // set selection mode from NONE -> SINGLE after a delay after it's been shown
        // this ensures selection mode is in SINGLE without having a selected row in the list
        list_box.connect_show(|list| {
            gtk::timeout_add(
                50,
                clone!(@weak list => @default-panic, move || {
                    list.set_selection_mode(SelectionMode::Single);
                    Continue(false)
                }),
            );
        });

        let feed_list = FeedList {
            list: list_box,
            scroll: sidebar_scroll.clone(),
            sender,
            categories: Arc::new(RwLock::new(HashMap::new())),
            feeds: Arc::new(RwLock::new(HashMap::new())),
            tree: Arc::new(RwLock::new(FeedListTree::new())),
            delayed_selection: Arc::new(RwLock::new(None)),
            hovered_category_expand: Arc::new(RwLock::new(None)),
            state: state.clone(),
        };
        feed_list.setup_dnd();
        feed_list
    }

    pub fn widget(&self) -> ListBox {
        self.list.clone()
    }

    fn clear_hovered_expand(hovered_category_expand: &Arc<RwLock<Option<(u32, CategoryID)>>>) {
        if hovered_category_expand.read().is_some() {
            if let Some((saved_source, _saved_id)) = &*hovered_category_expand.read() {
                GtkUtil::remove_source(Some(*saved_source));
            }
            *hovered_category_expand.write() = None;
        }
    }

    fn setup_dnd(&self) {
        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        self.list
            .drag_dest_set(DestDefaults::DROP | DestDefaults::MOTION, &[entry], DragAction::MOVE);
        self.list.drag_dest_add_text_targets();
        self.list.connect_drag_motion(clone!(
            @weak self.tree as tree,
            @weak self.feeds as feeds,
            @weak self.hovered_category_expand as hovered_category_expand,
            @weak self.categories as categories => @default-panic, move |widget, _drag_context, _x, y, _time|
        {
            // maybe we should keep track of the previous highlighted rows instead of iterating over all of them
            let children = widget.get_children();
            for widget in children {
                if let Some(ctx) = GtkUtil::get_dnd_style_context_widget(&widget) {
                    ctx.remove_class("drag-bottom");
                    ctx.remove_class("drag-top");
                    ctx.remove_class("drag-category");
                }
            }

            if let Some(row) = widget.get_row_at_y(y) {
                let alloc = row.get_allocation();
                let index = row.get_index();
                let is_category = GtkUtil::listboxrow_is_category(&row);
                let height_threshold = if is_category { 4 } else { 2 };

                if y <= alloc.y + (alloc.height / height_threshold) {
                    Self::clear_hovered_expand(&hovered_category_expand);
                    if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                        ctx.add_class("drag-top");
                        return Inhibit(false);
                    }
                }

                if is_category
                    && y >= alloc.y + (alloc.height / height_threshold)
                    && y <= alloc.y + (alloc.height * 3 / 4)
                {
                    if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                        ctx.add_class("drag-category");
                    }

                    // expand/collapse category on 1.2s hover
                    let hover = tree.read().calculate_selection(index);
                    if let Some(hovered_item) = hover {
                        if let (FeedListItemID::Category(id), _title) = hovered_item {
                            let mut start_hover = false;
                            if let Some((saved_source, saved_id)) = &*hovered_category_expand.read() {
                                if saved_id != &id {
                                    GtkUtil::remove_source(Some(*saved_source));
                                    start_hover = true;
                                }
                            } else {
                                start_hover = true;
                            }

                            if start_hover {
                                *hovered_category_expand.write() = Some((
                                    gtk::timeout_add(1200, clone!(
                                        @strong id,
                                        @weak tree,
                                        @weak categories,
                                        @weak hovered_category_expand,
                                        @weak feeds => @default-panic, move ||
                                    {
                                        if let Some(category_row) = categories.read().get(&id) {
                                            category_row.write().expand_collapse_arrow();
                                            Self::expand_collapse_category(&id, &tree, &categories, &feeds);
                                        }
                                        *hovered_category_expand.write() = None;
                                        Continue(false)
                                    }))
                                    .to_glib(),
                                    id,
                                ));
                            }
                        }
                    }

                    return Inhibit(false);
                }

                Self::clear_hovered_expand(&hovered_category_expand);

                // check next visible item
                let next_item = tree.write().calculate_next_item(index);
                if let SidebarIterateItem::SelectFeedListCategory(id) = &next_item {
                    if let Some(category_row) = categories.read().get(&id) {
                        if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&category_row.read().widget())
                        {
                            ctx.add_class("drag-top");
                            return Inhibit(false);
                        }
                    }
                } else if let SidebarIterateItem::SelectFeedListFeed(id) = &next_item {
                    if let Some(feed_row) = feeds.read().get(&id) {
                        if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&feed_row.read().widget()) {
                            ctx.add_class("drag-top");
                            return Inhibit(false);
                        }
                    }
                }

                // row after doesn't exist -> insert at last pos
                if let Some(ctx) = GtkUtil::get_dnd_style_context_listboxrow(&row) {
                    ctx.add_class("drag-bottom");
                    return Inhibit(false);
                }
            }

            Inhibit(false)
        }));
        self.list.connect_drag_leave(clone!(
            @weak self.hovered_category_expand as hovered_category_expand => @default-panic, move |widget, _drag_context, _time|
        {
            Self::clear_hovered_expand(&hovered_category_expand);
            let children = widget.get_children();
            for widget in children {
                if let Some(ctx) = GtkUtil::get_dnd_style_context_widget(&widget) {
                    ctx.remove_class("drag-bottom");
                    ctx.remove_class("drag-top");
                }
            }
        }));
        self.list.connect_drag_drop(|widget, drag_context, _x, _y, time| {
            if let Some(target_type) = drag_context.list_targets().get(0) {
                widget.drag_get_data(drag_context, target_type, time);
                return Inhibit(true);
            }
            Inhibit(false)
        });
        self.list.connect_drag_data_received(clone!(
            @weak self.tree as tree,
            @weak self.hovered_category_expand as hovered_category_expand,
            @strong self.sender as sender => @default-panic, move |widget, _ctx, _x, y, selection_data, _info, _time| {
            Self::clear_hovered_expand(&hovered_category_expand);
            let children = widget.get_children();
            for widget in children {
                if let Some(ctx) = GtkUtil::get_dnd_style_context_widget(&widget) {
                    ctx.remove_class("drag-bottom");
                    ctx.remove_class("drag-top");
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

                if let Ok((parent_category, sort_index)) = tree.read().calculate_dnd(index).map_err(|_| {
                    error!("Failed to calculate Drag&Drop action");
                }) {
                    if let Some(dnd_data_string) = selection_data.get_text() {
                        if dnd_data_string.contains("FeedID") {
                            let dnd_data_string = dnd_data_string.as_str().to_owned().split_off(6);
                            let dnd_data_string: Vec<&str> = dnd_data_string.split(";").collect();
                            let feed_string =
                                dnd_data_string.get(0).expect("Didn't receive feed ID with DnD data.");
                            let feed: FeedID =
                                serde_json::from_str(feed_string).expect("Failed to deserialize FeedID.");
                            let category_string = dnd_data_string
                                .get(1)
                                .expect("Didn't receive category ID with DnD data.");
                            let current_category: CategoryID =
                                serde_json::from_str(category_string).expect("Failed to deserialize FeedID.");
                            let dnd_data = FeedListDndAction::MoveFeed(
                                feed,
                                current_category,
                                parent_category.clone(),
                                sort_index,
                            );
                            Util::send(&sender, Action::DragAndDrop(dnd_data));
                        }

                        if dnd_data_string.contains("CategoryID") {
                            let category: CategoryID =
                                serde_json::from_str(&dnd_data_string.as_str().to_owned().split_off(10))
                                    .expect("Failed to deserialize CategoryID.");
                            let dnd_data =
                                FeedListDndAction::MoveCategory(category, parent_category.clone(), sort_index);
                            Util::send(&sender, Action::DragAndDrop(dnd_data));
                        }
                    }
                }
            }
        }));
    }

    pub fn update(&mut self, new_tree: FeedListTree, features: &Arc<RwLock<Option<PluginCapabilities>>>) {
        let mut old_tree = new_tree;
        std::mem::swap(&mut old_tree, &mut *self.tree.write());

        Util::serialize_and_save(&old_tree, "old_tree.json").expect("debug printing");
        Util::serialize_and_save(&*self.tree.read(), "new_tree.json").expect("debug printing");

        let tree_diff = old_tree.generate_diff(&mut self.tree.write());
        for diff in tree_diff {
            match diff {
                FeedListChangeSet::RemoveFeed(feed_id) => {
                    if let Some(feed_handle) = self.feeds.read().get(&feed_id) {
                        self.list.remove(&feed_handle.read().widget());
                    }
                    self.feeds.write().remove(&feed_id);
                }
                FeedListChangeSet::RemoveCategory(category_id) => {
                    if let Some(category_handle) = self.categories.read().get(&category_id) {
                        self.list.remove(&category_handle.read().widget());
                    }
                    self.categories.write().remove(&category_id);
                }
                FeedListChangeSet::AddFeed(model, pos, visible) => {
                    self.add_feed(&model, pos, visible, features);
                }
                FeedListChangeSet::AddCategory(model, pos, visible) => {
                    self.add_category(&model, pos, visible, features);
                }
                FeedListChangeSet::FeedUpdateItemCount(id, count) => {
                    if let Some(feed_handle) = self.feeds.read().get(&id) {
                        feed_handle.read().update_item_count(count);
                    }
                }
                FeedListChangeSet::CategoryUpdateItemCount(id, count) => {
                    if let Some(category_handle) = self.categories.read().get(&id) {
                        category_handle.read().update_item_count(count);
                    }
                }
                FeedListChangeSet::FeedUpdateLabel(id, label) => {
                    if let Some(feed_handle) = self.feeds.read().get(&id) {
                        feed_handle.read().update_title(&label);
                    }
                }
                FeedListChangeSet::CategoryUpdateLabel(id, label) => {
                    if let Some(category_handle) = self.categories.read().get(&id) {
                        category_handle.read().update_title(&label);
                    }
                }
            }
        }
    }

    fn add_category(
        &mut self,
        category: &FeedListCategoryModel,
        pos: i32,
        visible: bool,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
    ) {
        let category_widget = CategoryRow::new(category, &self.state, features, visible, self.sender.clone());
        self.list.insert(&category_widget.read().widget(), pos);
        self.categories
            .write()
            .insert(category.id.clone(), category_widget.clone());

        category_widget
            .read()
            .expander_event()
            .connect_button_press_event(clone!(
                @weak self.feeds as feeds,
                @weak self.categories as categories,
                @strong category.id as category_id,
                @weak self.tree as tree => @default-panic, move |_widget, event|
            {
                if event.get_button() != 1 {
                    return Inhibit(false);
                }
                match event.get_event_type() {
                    EventType::ButtonPress => (),
                    _ => return Inhibit(false),
                }
                Self::expand_collapse_category(&category_id, &tree, &categories, &feeds);
                Inhibit(true)
            }));
    }

    pub fn expand_collapse_selected_category(&self) {
        if let Some(row) = self.list.get_selected_row() {
            let selection = self.tree.read().calculate_selection(row.get_index());
            if let Some(selected_item) = selection {
                if let (FeedListItemID::Category(id), _title) = selected_item {
                    if let Some(category_row) = self.categories.read().get(&id) {
                        category_row.write().expand_collapse_arrow();
                        Self::expand_collapse_category(&id, &self.tree, &self.categories, &self.feeds);
                    }
                }
            }
        }
    }

    fn expand_collapse_category(
        category_id: &CategoryID,
        tree: &Arc<RwLock<FeedListTree>>,
        categories: &Arc<RwLock<HashMap<CategoryID, Arc<RwLock<CategoryRow>>>>>,
        feeds: &Arc<RwLock<HashMap<FeedID, Arc<RwLock<FeedRow>>>>>,
    ) {
        if let Some((feed_ids, category_ids, expaneded)) = tree.write().collapse_expand_category(category_id) {
            for feed_id in feed_ids {
                if let Some(feed_handle) = feeds.read().get(&feed_id) {
                    if expaneded {
                        feed_handle.write().expand();
                    } else {
                        feed_handle.write().collapse();
                    }
                }
            }
            for category_id in category_ids {
                if let Some(category_handle) = categories.read().get(&category_id) {
                    if expaneded {
                        category_handle.write().expand();
                    } else {
                        category_handle.write().collapse();
                    }
                }
            }
        }
    }

    fn add_feed(
        &mut self,
        feed: &FeedListFeedModel,
        pos: i32,
        visible: bool,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
    ) {
        let feed_widget = FeedRow::new(feed, &self.state, features, visible, self.sender.clone());
        self.list.insert(&feed_widget.read().widget(), pos);
        self.feeds.write().insert(feed.id.clone(), feed_widget);
    }

    pub fn deselect(&self) {
        self.list.unselect_all();
    }

    pub fn get_selection(&self) -> Option<(FeedListItemID, String)> {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.tree.read().calculate_selection(index);
        }
        None
    }

    pub fn get_first_item(&self) -> Option<FeedListItemID> {
        self.tree.read().top_level.first().map(|item| match item {
            FeedListItem::Feed(item) => FeedListItemID::Feed(item.id.clone()),
            FeedListItem::Category(item) => FeedListItemID::Category(item.id.clone()),
        })
    }

    pub fn get_last_item(&self, last_item: Option<FeedListItem>) -> Option<FeedListItemID> {
        let last_item = if last_item.is_some() {
            last_item
        } else {
            self.tree.read().top_level.last().cloned()
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
            FeedListItemID::Category(category) => match self.categories.read().get(&category) {
                Some(category_row) => category_row.read().widget(),
                None => return Err(FeedListErrorKind::CategoryNotFound.into()),
            },
            FeedListItemID::Feed(feed) => match self.feeds.read().get(&feed) {
                Some(feed_row) => feed_row.read().widget(),
                None => return Err(FeedListErrorKind::FeedNotFound.into()),
            },
        };

        gtk::idle_add(clone!(
            @weak self.list as list,
            @weak self.delayed_selection as delayed_selection,
            @weak row => @default-panic, move ||
        {
            list.select_row(Some(&row));

            *delayed_selection.write() = Some(
                gtk::timeout_add(300, clone!(
                    @weak delayed_selection,
                    @weak row => @default-panic, move ||
                {
                    row.emit_activate();
                    delayed_selection.write().take();
                    Continue(false)
                }))
                .to_glib(),
            );

            Continue(false)
        }));

        Ok(())
    }

    pub fn cancel_selection(&self) {
        GtkUtil::remove_source(*self.delayed_selection.read());
        *self.delayed_selection.write() = None;
    }

    pub fn select_next_item(&self) -> SidebarIterateItem {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.tree.write().calculate_next_item(index);
        }
        SidebarIterateItem::NothingSelected
    }

    pub fn select_prev_item(&self) -> SidebarIterateItem {
        if let Some(row) = self.list.get_selected_row() {
            let index = row.get_index();
            return self.tree.write().calculate_prev_item(index);
        }
        SidebarIterateItem::NothingSelected
    }

    pub fn update_offline(&self) {
        for (_key, value) in self.feeds.read().iter() {
            if self.state.read().get_offline() {
                value.read().disable_dnd();
            } else {
                value.read().enable_dnd();
            }
        }
    }
}
