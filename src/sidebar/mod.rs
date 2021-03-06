mod error;
mod feed_list;
mod footer;
pub mod models;
mod tag_list;

use self::error::{SidebarError, SidebarErrorKind};
use self::footer::SidebarFooter;
use crate::app::Action;
use crate::i18n::i18n;
use crate::main_window_state::MainWindowState;
use crate::util::{BuilderHelper, GtkUtil, Util};
use failure::ResultExt;
pub use feed_list::models::{FeedListDndAction, FeedListItemID, FeedListTree};
use feed_list::FeedList;
use gdk::{EventMask, EventType};
use glib::{clone, source::Continue, translate::ToGlib, Sender};
use gtk::{
    prelude::WidgetExtManual, Box, BoxExt, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, ListBoxExt, Revealer,
    RevealerExt, ScrolledWindow, StyleContextExt, WidgetExt,
};
pub use models::SidebarIterateItem;
use models::SidebarSelection;
use news_flash::models::{PluginCapabilities, PluginID, PluginIcon};
use news_flash::NewsFlash;
use parking_lot::RwLock;
use std::sync::Arc;
pub use tag_list::models::TagListModel;
use tag_list::TagList;

#[derive(Clone, Debug)]
pub struct SideBar {
    sender: Sender<Action>,
    state: Arc<RwLock<MainWindowState>>,
    sidebar: Box,
    tags_box: Box,
    logo: Image,
    all_event_box: EventBox,
    all_label: Label,
    item_count: i64,
    service_label: Label,
    scale_factor: i32,
    pub feed_list: Arc<RwLock<FeedList>>,
    pub tag_list: Arc<RwLock<TagList>>,
    selection: Arc<RwLock<SidebarSelection>>,
    categories_expander: Image,
    tags_expander: Image,
    categories_revealer: Revealer,
    tags_revealer: Revealer,
    expanded_categories: Arc<RwLock<bool>>,
    expanded_tags: Arc<RwLock<bool>>,
    delayed_all_selection: Arc<RwLock<Option<u32>>>,
    pub footer: Arc<SidebarFooter>,
}

impl SideBar {
    pub fn new(
        state: &Arc<RwLock<MainWindowState>>,
        sender: Sender<Action>,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
    ) -> Self {
        let builder = BuilderHelper::new("sidebar");

        let sidebar = builder.get::<Box>("toplevel");
        let tags_box = builder.get::<Box>("tags");
        let logo = builder.get::<Image>("logo");
        let all_label = builder.get::<Label>("unread_count_all");
        let item_count = 0;
        let service_label = builder.get::<Label>("service_label");
        let categories_event_box = builder.get::<EventBox>("categories_event_box");
        let categories_expander = builder.get::<Image>("categories_expander");
        let tags_event_box = builder.get::<EventBox>("tags_event_box");
        let tags_expander = builder.get::<Image>("tags_expander");
        let categories_revealer = builder.get::<Revealer>("categories_revealer");
        let tags_revealer = builder.get::<Revealer>("tags_revealer");
        let all_event_box = builder.get::<EventBox>("all_event_box");
        let feed_list_box = builder.get::<Box>("feed_list_box");
        let tag_list_box = builder.get::<Box>("tags_list_box");
        let sidebar_scroll = builder.get::<ScrolledWindow>("sidebar_scroll");

        let selection_handle = Arc::new(RwLock::new(SidebarSelection::All));
        let delayed_all_selection = Arc::new(RwLock::new(None));

        let feed_list = FeedList::new(&sidebar_scroll, state, sender.clone());
        let tag_list = TagList::new(state);
        let footer = Arc::new(SidebarFooter::new(
            &builder,
            state,
            &sender,
            features,
            &selection_handle,
        ));

        let feed_list_handle = Arc::new(RwLock::new(feed_list));
        let tag_list_handle = Arc::new(RwLock::new(tag_list));

        feed_list_box.pack_start(&feed_list_handle.read().widget(), false, true, 0);
        tag_list_box.pack_start(&tag_list_handle.read().widget(), false, true, 0);

        feed_list_handle.read().widget().connect_row_activated(
            clone!(@strong sender, @weak footer, @weak selection_handle => @default-panic, move |_list, _row| {
                Util::send(
                    &sender,
                    Action::SidebarSelection((*selection_handle.read()).clone()),
                );
                footer.update();
            }),
        );

        feed_list_handle.read().widget().connect_row_selected(clone!(
            @weak all_event_box,
            @weak tag_list_handle,
            @strong feed_list_handle as self_handle,
            @strong selection_handle,
            @weak delayed_all_selection => @default-panic, move |_list, row|
        {
            // do nothing if selection was cleared
            if row.is_none() {
                return;
            }
            // deselect 'all' & tag_list
            Self::deselect_all_button(&all_event_box, &delayed_all_selection);
            tag_list_handle.read().deselect();

            if let Some((item, title)) = self_handle.read().get_selection() {
                let selection = SidebarSelection::from_feed_list_selection(item, title);
                *selection_handle.write() = selection;
            }
        }));

        tag_list_handle.read().widget().connect_row_activated(
            clone!(@weak selection_handle, @weak footer, @strong sender => @default-panic, move |_list, _row| {
                Util::send(
                    &sender,
                    Action::SidebarSelection((*selection_handle.read()).clone()),
                );
                footer.update();
            }),
        );

        tag_list_handle.read().widget().connect_row_selected(clone!(
            @weak all_event_box,
            @weak feed_list_handle,
            @strong tag_list_handle,
            @weak selection_handle,
            @weak delayed_all_selection => @default-panic, move |_list, row| {
            // do nothing if selection was cleared
            if row.is_none() {
                return;
            }
            // deselect 'all' & tag_list
            Self::deselect_all_button(&all_event_box, &delayed_all_selection);
            feed_list_handle.read().deselect();

            if let Some((selected_id, title)) = tag_list_handle.read().get_selection() {
                let selection = SidebarSelection::Tag(selected_id, title);
                *selection_handle.write() = selection;
            }
        }));

        let scale = GtkUtil::get_scale(&sidebar);

        let expanded_categories = Arc::new(RwLock::new(true));
        let expanded_tags = Arc::new(RwLock::new(false));

        Self::setup_expander(
            &categories_event_box,
            &categories_expander,
            &categories_revealer,
            &expanded_categories,
        );
        Self::setup_expander(&tags_event_box, &tags_expander, &tags_revealer, &expanded_tags);
        Self::setup_all_button(
            &all_event_box,
            &sender,
            feed_list_handle.clone(),
            tag_list_handle.clone(),
            selection_handle.clone(),
            footer.clone(),
            &delayed_all_selection,
        );

        SideBar {
            sender,
            state: state.clone(),
            sidebar,
            tags_box,
            logo,
            all_event_box,
            all_label,
            item_count,
            service_label,
            scale_factor: scale,
            feed_list: feed_list_handle,
            tag_list: tag_list_handle,
            selection: selection_handle,
            categories_expander,
            tags_expander,
            categories_revealer,
            tags_revealer,
            expanded_categories,
            expanded_tags,
            delayed_all_selection,
            footer,
        }
    }

    pub fn widget(&self) -> Box {
        self.sidebar.clone()
    }

    pub fn update_feedlist(&mut self, tree: FeedListTree, features: &Arc<RwLock<Option<PluginCapabilities>>>) {
        self.feed_list.write().update(tree, features);
        self.sidebar.show_all();
    }

    pub fn update_taglist(&mut self, list: TagListModel) {
        self.tag_list.write().update(list);
        self.sidebar.show_all();
    }

    pub fn hide_taglist(&self) {
        self.tags_box.hide();
    }

    pub fn show_taglist(&self) {
        self.tags_box.show_all();
        self.tag_list.read().widget().show_all();
        self.tags_box.show();
    }

    pub fn update_all(&mut self, item_count: i64) {
        self.item_count = item_count;
        self.update_all_label();
    }

    fn update_all_label(&self) {
        self.all_label.set_text(&format!("{}", self.get_count_all()));
    }

    pub fn get_count_all(&self) -> i64 {
        self.item_count
    }

    pub fn set_service(&self, id: Option<&PluginID>, user_name: Option<String>) -> Result<(), SidebarError> {
        let generic_icon = GtkUtil::create_surface_from_icon_name("feed-service-generic", 64, self.scale_factor);
        let generic_user = i18n("Unitialized");

        if let Some(id) = id {
            let list = NewsFlash::list_backends();
            if let Some(info) = list.get(id) {
                if let Some(icon) = &info.icon_symbolic {
                    let surface = match icon {
                        PluginIcon::Vector(icon) => {
                            GtkUtil::create_surface_from_bytes(&icon.data, icon.width, icon.height, self.scale_factor)
                                .context(SidebarErrorKind::MetaData)?
                        }
                        PluginIcon::Pixel(icon) => GtkUtil::create_surface_from_pixelicon(icon, self.scale_factor)
                            .context(SidebarErrorKind::MetaData)?,
                    };
                    self.logo.set_from_surface(Some(&surface));
                } else {
                    self.logo.set_from_surface(Some(&generic_icon));
                }

                if let Some(user_name) = user_name {
                    self.service_label.set_text(&user_name);
                } else {
                    self.service_label.set_text(&info.name);
                }
            } else {
                self.logo.set_from_surface(Some(&generic_icon));
                self.service_label.set_text(&generic_user);
            }
        } else {
            self.logo.set_from_surface(Some(&generic_icon));
            self.service_label.set_text(&generic_user);
        }

        Ok(())
    }

    fn setup_expander(event_box: &EventBox, expander: &Image, revealer: &Revealer, expanded: &Arc<RwLock<bool>>) {
        event_box.set_events(EventMask::BUTTON_PRESS_MASK);
        event_box.set_events(EventMask::ENTER_NOTIFY_MASK);
        event_box.set_events(EventMask::LEAVE_NOTIFY_MASK);
        event_box.connect_enter_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.add_class("highlight");
            Inhibit(false)
        });
        event_box.connect_leave_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.remove_class("highlight");
            Inhibit(false)
        });

        event_box.connect_button_press_event(clone!(
            @weak expander,
            @weak expanded,
            @weak revealer => @default-panic, move |_widget, event| {
            if event.get_event_type() == EventType::ButtonPress {
                if event.get_button() != 1 {
                    return Inhibit(false);
                }
                match event.get_event_type() {
                    EventType::ButtonPress => (),
                    _ => return Inhibit(false),
                }
                let is_expanded = *expanded.read();
                Self::expand_list(!is_expanded, &revealer, &expander, &expanded);
            }
            Inhibit(false)
        }));
    }

    fn expand_list(expand: bool, revealer: &Revealer, expander: &Image, expanded: &Arc<RwLock<bool>>) {
        let context = expander.get_style_context();
        if expand {
            context.remove_class("backward-arrow-collapsed");
            context.add_class("backward-arrow-expanded");
            revealer.set_reveal_child(true);
        } else {
            context.add_class("backward-arrow-collapsed");
            context.remove_class("backward-arrow-expanded");
            revealer.set_reveal_child(false);
        }
        *expanded.write() = expand;
    }

    fn setup_all_button(
        event_box: &EventBox,
        sender: &Sender<Action>,
        feed_list_handle: Arc<RwLock<FeedList>>,
        tag_list_handle: Arc<RwLock<TagList>>,
        selection_handle: Arc<RwLock<SidebarSelection>>,
        footer: Arc<SidebarFooter>,
        delayed_selection: &Arc<RwLock<Option<u32>>>,
    ) {
        let context = event_box.get_style_context();
        context.add_class("selected");
        event_box.set_events(EventMask::BUTTON_PRESS_MASK);
        event_box.set_events(EventMask::ENTER_NOTIFY_MASK);
        event_box.set_events(EventMask::LEAVE_NOTIFY_MASK);
        event_box.connect_enter_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.add_class("highlight");
            Inhibit(false)
        });
        event_box.connect_leave_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.remove_class("highlight");
            Inhibit(false)
        });

        event_box.connect_button_press_event(clone!(
            @strong sender,
            @weak footer,
            @weak delayed_selection => @default-panic, move |widget, event| {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return Inhibit(false),
            }

            feed_list_handle.read().deselect();
            tag_list_handle.read().deselect();

            Self::select_all_button(widget, &sender, &selection_handle, &delayed_selection);
            footer.update();
            Inhibit(false)
        }));
    }

    pub fn select_all_button_no_update(&self) {
        *self.selection.write() = SidebarSelection::All;
        GtkUtil::remove_source(*self.delayed_all_selection.read());
        let context = self.all_event_box.get_style_context();
        context.add_class("selected");
    }

    fn select_all_button(
        all_event_box: &EventBox,
        sender: &Sender<Action>,
        selection_handle: &Arc<RwLock<SidebarSelection>>,
        delayed_selection: &Arc<RwLock<Option<u32>>>,
    ) {
        *selection_handle.write() = SidebarSelection::All;
        let context = all_event_box.get_style_context();
        context.add_class("selected");

        GtkUtil::remove_source(*delayed_selection.read());
        *delayed_selection.write() = Some(
            gtk::timeout_add(
                50,
                clone!(
                    @strong sender, @weak delayed_selection as source_id => @default-panic, move || {
                    Util::send(&sender, Action::SidebarSelection(SidebarSelection::All));
                    *source_id.write() = None;
                    Continue(false)
                }),
            )
            .to_glib(),
        );
    }

    fn deselect_all_button(all_event_box: &EventBox, delayed_selection: &Arc<RwLock<Option<u32>>>) {
        let context = all_event_box.get_style_context();
        context.remove_class("selected");
        GtkUtil::remove_source(*delayed_selection.read());
        *delayed_selection.write() = None;
    }

    pub fn select_next_item(&self) -> Result<(), SidebarError> {
        let select_next = match *self.selection.read() {
            SidebarSelection::All => SidebarIterateItem::FeedListSelectFirstItem,
            SidebarSelection::Category(_, _) | SidebarSelection::Feed(_, _, _) => {
                self.feed_list.read().select_next_item()
            }
            SidebarSelection::Tag(_, _) => self.tag_list.read().get_next_item(),
        };
        self.select_item(select_next)
    }

    pub fn select_prev_item(&self) -> Result<(), SidebarError> {
        let select_next = match *self.selection.read() {
            SidebarSelection::All => SidebarIterateItem::TagListSelectLastItem,
            SidebarSelection::Category(_, _) | SidebarSelection::Feed(_, _, _) => {
                self.feed_list.read().select_prev_item()
            }
            SidebarSelection::Tag(_, _) => self.tag_list.read().get_prev_item(),
        };
        self.select_item(select_next)
    }

    fn select_item(&self, selection: SidebarIterateItem) -> Result<(), SidebarError> {
        self.deselect();

        match selection {
            SidebarIterateItem::SelectAll => {
                Self::select_all_button(
                    &self.all_event_box,
                    &self.sender,
                    &self.selection,
                    &self.delayed_all_selection,
                );
            }
            SidebarIterateItem::SelectFeedListFeed(id, parent_id) => {
                self.feed_list
                    .read()
                    .set_selection(FeedListItemID::Feed(id, parent_id))
                    .context(SidebarErrorKind::Selection)?;
            }
            SidebarIterateItem::SelectFeedListCategory(id) => {
                self.feed_list
                    .read()
                    .set_selection(FeedListItemID::Category(id))
                    .context(SidebarErrorKind::Selection)?;
            }
            SidebarIterateItem::FeedListSelectFirstItem => {
                Self::expand_list(
                    true,
                    &self.categories_revealer,
                    &self.categories_expander,
                    &self.expanded_categories,
                );
                if let Some(item) = self.feed_list.read().get_first_item() {
                    self.feed_list
                        .read()
                        .set_selection(item)
                        .context(SidebarErrorKind::Selection)?;
                }
            }
            SidebarIterateItem::FeedListSelectLastItem => {
                Self::expand_list(
                    true,
                    &self.categories_revealer,
                    &self.categories_expander,
                    &self.expanded_categories,
                );
                if let Some(item) = self.feed_list.read().get_last_item(None) {
                    self.feed_list
                        .read()
                        .set_selection(item)
                        .context(SidebarErrorKind::Selection)?;
                }
            }
            SidebarIterateItem::SelectTagList(id) => {
                self.tag_list
                    .read()
                    .set_selection(id)
                    .context(SidebarErrorKind::Selection)?;
            }
            SidebarIterateItem::TagListSelectFirstItem => {
                // if tags not supported or not available jump back to "All Articles"
                if !self.tags_box.is_visible() {
                    return self.select_item(SidebarIterateItem::SelectAll);
                }
                Self::expand_list(true, &self.tags_revealer, &self.tags_expander, &self.expanded_tags);
                if let Some(item) = self.tag_list.read().get_first_item() {
                    self.tag_list
                        .read()
                        .set_selection(item)
                        .context(SidebarErrorKind::Selection)?;
                }
            }
            SidebarIterateItem::TagListSelectLastItem => {
                // if tags not supported or not available jump back to "All Articles"
                if !self.tags_box.is_visible() {
                    return self.select_item(SidebarIterateItem::FeedListSelectLastItem);
                }
                Self::expand_list(true, &self.tags_revealer, &self.tags_expander, &self.expanded_tags);
                if let Some(item) = self.tag_list.read().get_last_item() {
                    self.tag_list
                        .read()
                        .set_selection(item)
                        .context(SidebarErrorKind::Selection)?;
                }
            }
            SidebarIterateItem::NothingSelected => { /* nothing */ }
        }
        Ok(())
    }

    pub fn get_selection(&self) -> SidebarSelection {
        (*self.selection.read()).clone()
    }

    fn deselect(&self) {
        Self::deselect_all_button(&self.all_event_box, &self.delayed_all_selection);
        self.feed_list.read().cancel_selection();
        self.feed_list.read().widget().unselect_all();
        self.tag_list.read().cancel_selection();
        self.tag_list.read().widget().unselect_all();
    }

    pub fn expand_collapse_selected_category(&self) {
        self.feed_list.read().expand_collapse_selected_category()
    }
}
