mod error;
mod feed_list;
mod footer;
pub mod models;
mod tag_list;

use self::error::{SidebarError, SidebarErrorKind};
use self::footer::SidebarFooter;
use crate::app::Action;
use crate::util::{BuilderHelper, GtkUtil, Util};
use failure::ResultExt;
pub use feed_list::models::{FeedListDndAction, FeedListItemID, FeedListTree};
use feed_list::FeedList;
use gdk::{EventMask, EventType};
use glib::{translate::ToGlib, Sender};
use gtk::{
    Box, BoxExt, Button, Continue, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, ListBoxExt, Revealer,
    RevealerExt, ScrolledWindow, StyleContextExt, WidgetExt, WidgetExtManual,
};
pub use models::SidebarIterateItem;
use models::SidebarSelection;
use news_flash::models::{PluginID, PluginIcon};
use news_flash::NewsFlash;
use std::sync::Arc;
use parking_lot::RwLock;
pub use tag_list::models::TagListModel;
use tag_list::TagList;

#[derive(Clone, Debug)]
pub struct SideBar {
    sender: Sender<Action>,
    sidebar: Box,
    tags_box: Box,
    logo: Image,
    all_event_box: EventBox,
    all_label: Label,
    item_count: i64,
    service_label: Label,
    scale_factor: i32,
    feed_list: Arc<RwLock<FeedList>>,
    tag_list: Arc<RwLock<TagList>>,
    selection: Arc<RwLock<SidebarSelection>>,
    categories_expander: Image,
    tags_expander: Image,
    categories_revealer: Revealer,
    tags_revealer: Revealer,
    expanded_categories: Arc<RwLock<bool>>,
    expanded_tags: Arc<RwLock<bool>>,
    delayed_all_selection: Arc<RwLock<Option<u32>>>,
    footer: Arc<RwLock<SidebarFooter>>,
}

impl SideBar {
    pub fn new(sender: Sender<Action>) -> Self {
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

        let feed_list = FeedList::new(&sidebar_scroll, sender.clone());
        let tag_list = TagList::new();
        let footer = SidebarFooter::new(&builder, &sender);

        let feed_list_handle = Arc::new(RwLock::new(feed_list));
        let tag_list_handle = Arc::new(RwLock::new(tag_list));
        let footer_handle = Arc::new(RwLock::new(footer));
        let selection_handle = Arc::new(RwLock::new(SidebarSelection::All));
        let delayed_all_selection = Arc::new(RwLock::new(None));

        feed_list_box.pack_start(&feed_list_handle.read().widget(), false, true, 0);
        tag_list_box.pack_start(&tag_list_handle.read().widget(), false, true, 0);

        let feed_list_selection_handle = selection_handle.clone();
        let sender_clone = sender.clone();
        feed_list_handle
            .read()
            .widget()
            .connect_row_activated(move |_list, _row| {
                Util::send(
                    &sender_clone,
                    Action::SidebarSelection((*feed_list_selection_handle.read()).clone()),
                );
            });

        let feed_list_all_event_box = all_event_box.clone();
        let feed_list_tag_list_handle = tag_list_handle.clone();
        let feed_list_feed_list_handle = feed_list_handle.clone();
        let feed_list_selection_handle = selection_handle.clone();
        let feed_list_delayed_all_selection = delayed_all_selection.clone();
        let feed_list_footer_handle = footer_handle.clone();
        feed_list_handle
            .read()
            .widget()
            .connect_row_selected(move |_list, row| {
                // do nothing if selection was cleared
                if row.is_none() {
                    return;
                }
                // deselect 'all' & tag_list
                Self::deselect_all_button(&feed_list_all_event_box, &feed_list_delayed_all_selection);
                feed_list_footer_handle.read().set_remove_button_sensitive(true);
                feed_list_tag_list_handle.read().deselect();

                if let Some((item, title)) = feed_list_feed_list_handle.read().get_selection() {
                    let selection = SidebarSelection::from_feed_list_selection(item, title);
                    *feed_list_selection_handle.write() = selection.clone();
                }
            });

        let tag_list_selection_handle = selection_handle.clone();
        let sender_clone = sender.clone();
        tag_list_handle
            .read()
            .widget()
            .connect_row_activated(move |_list, _row| {
                Util::send(
                    &sender_clone,
                    Action::SidebarSelection((*tag_list_selection_handle.read()).clone()),
                );
            });

        let tag_list_all_event_box = all_event_box.clone();
        let tag_list_feed_list_handle = feed_list_handle.clone();
        let tag_list_tag_list_handle = tag_list_handle.clone();
        let tag_list_selection_handle = selection_handle.clone();
        let tag_list_delayed_all_selection = delayed_all_selection.clone();
        let tag_list_footer_handle = footer_handle.clone();
        tag_list_handle
            .read()
            .widget()
            .connect_row_selected(move |_list, row| {
                // do nothing if selection was cleared
                if row.is_none() {
                    return;
                }
                // deselect 'all' & tag_list
                Self::deselect_all_button(&tag_list_all_event_box, &tag_list_delayed_all_selection);
                tag_list_footer_handle.read().set_remove_button_sensitive(true);
                tag_list_feed_list_handle.read().deselect();

                if let Some(selected_id) = tag_list_tag_list_handle.read().get_selection() {
                    let selection = SidebarSelection::Tag(selected_id);
                    *tag_list_selection_handle.write() = selection.clone();
                }
            });

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
            footer_handle.clone(),
            &delayed_all_selection,
        );

        SideBar {
            sender,
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
            footer: footer_handle,
        }
    }

    pub fn widget(&self) -> Box {
        self.sidebar.clone()
    }

    pub fn update_feedlist(&mut self, tree: FeedListTree) {
        self.feed_list.write().update(tree);
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

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), SidebarError> {
        let list = NewsFlash::list_backends();
        let info = list
            .get(id)
            .ok_or_else(|| SidebarErrorKind::UnknownPlugin(id.clone()))?;
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
            let surface = GtkUtil::create_surface_from_icon_name("feed-service-generic", 64, self.scale_factor);
            self.logo.set_from_surface(Some(&surface));
        }

        if let Some(user_name) = user_name {
            self.service_label.set_text(&user_name);
        } else {
            self.service_label.set_text(&info.name);
        }

        Ok(())
    }

    fn setup_expander(event_box: &EventBox, expander: &Image, revealer: &Revealer, expanded: &Arc<RwLock<bool>>) {
        let expander = expander.clone();
        let expanded = expanded.clone();
        let revealer = revealer.clone();
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

        event_box.connect_button_press_event(move |_widget, event| {
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
        });
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
        footer_handle: Arc<RwLock<SidebarFooter>>,
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

        let delayed_selection = delayed_selection.clone();
        let sender = sender.clone();
        event_box.connect_button_press_event(move |widget, event| {
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
            footer_handle.read().set_remove_button_sensitive(false);
            Inhibit(false)
        });
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
        let source_id = delayed_selection.clone();
        let sender = sender.clone();
        *delayed_selection.write() = Some(
            gtk::timeout_add(300, move || {
                let sender = sender.clone();
                gtk::idle_add(move || {
                    Util::send(&sender, Action::SidebarSelection(SidebarSelection::All));
                    Continue(false)
                });

                *source_id.write() = None;
                Continue(false)
            })
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
            SidebarSelection::Cateogry(_) | SidebarSelection::Feed(_) => self.feed_list.read().select_next_item(),
            SidebarSelection::Tag(_) => self.tag_list.read().get_next_item(),
        };
        self.select_item(select_next)
    }

    pub fn select_prev_item(&self) -> Result<(), SidebarError> {
        let select_next = match *self.selection.read() {
            SidebarSelection::All => SidebarIterateItem::TagListSelectLastItem,
            SidebarSelection::Cateogry(_) | SidebarSelection::Feed(_) => self.feed_list.read().select_prev_item(),
            SidebarSelection::Tag(_) => self.tag_list.read().get_prev_item(),
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
            SidebarIterateItem::SelectFeedListFeed(id) => {
                self.feed_list
                    .read()
                    .set_selection(FeedListItemID::Feed(id))
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

    pub fn get_add_button(&self) -> Button {
        self.footer.read().get_add_button()
    }
}
