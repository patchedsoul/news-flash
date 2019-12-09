mod error;
mod feed_list;
mod footer;
pub mod models;
mod tag_list;

use self::error::{SidebarError, SidebarErrorKind};
use self::footer::SidebarFooter;
use crate::app::Action;
use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
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
use std::cell::RefCell;
use std::rc::Rc;
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
    feed_list: GtkHandle<FeedList>,
    tag_list: GtkHandle<TagList>,
    selection: GtkHandle<SidebarSelection>,
    categories_expander: Image,
    tags_expander: Image,
    categories_revealer: Revealer,
    tags_revealer: Revealer,
    expanded_categories: GtkHandle<bool>,
    expanded_tags: GtkHandle<bool>,
    delayed_all_selection: GtkHandle<Option<u32>>,
    footer: GtkHandle<SidebarFooter>,
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
        let footer = SidebarFooter::new(&builder);

        let feed_list_handle = gtk_handle!(feed_list);
        let tag_list_handle = gtk_handle!(tag_list);
        let footer_handle = gtk_handle!(footer);
        let selection_handle = gtk_handle!(SidebarSelection::All);
        let delayed_all_selection = gtk_handle!(None);

        feed_list_box.pack_start(&feed_list_handle.borrow().widget(), false, true, 0);
        tag_list_box.pack_start(&tag_list_handle.borrow().widget(), false, true, 0);

        let feed_list_selection_handle = selection_handle.clone();
        let sender_clone = sender.clone();
        feed_list_handle
            .borrow()
            .widget()
            .connect_row_activated(move |_list, _row| {
                GtkUtil::send(
                    &sender_clone,
                    Action::SidebarSelection((*feed_list_selection_handle.borrow()).clone()),
                );
            });

        let feed_list_all_event_box = all_event_box.clone();
        let feed_list_tag_list_handle = tag_list_handle.clone();
        let feed_list_feed_list_handle = feed_list_handle.clone();
        let feed_list_selection_handle = selection_handle.clone();
        let feed_list_delayed_all_selection = delayed_all_selection.clone();
        let feed_list_footer_handle = footer_handle.clone();
        feed_list_handle
            .borrow()
            .widget()
            .connect_row_selected(move |_list, row| {
                // do nothing if selection was cleared
                if row.is_none() {
                    return;
                }
                // deselect 'all' & tag_list
                Self::deselect_all_button(&feed_list_all_event_box, &feed_list_delayed_all_selection);
                feed_list_footer_handle.borrow().set_remove_button_sensitive(true);
                feed_list_tag_list_handle.borrow().deselect();

                if let Some((item, title)) = feed_list_feed_list_handle.borrow().get_selection() {
                    let selection = SidebarSelection::from_feed_list_selection(item, title);
                    *feed_list_selection_handle.borrow_mut() = selection.clone();
                }
            });

        let tag_list_selection_handle = selection_handle.clone();
        let sender_clone = sender.clone();
        tag_list_handle
            .borrow()
            .widget()
            .connect_row_activated(move |_list, _row| {
                GtkUtil::send(
                    &sender_clone,
                    Action::SidebarSelection((*tag_list_selection_handle.borrow()).clone()),
                );
            });

        let tag_list_all_event_box = all_event_box.clone();
        let tag_list_feed_list_handle = feed_list_handle.clone();
        let tag_list_tag_list_handle = tag_list_handle.clone();
        let tag_list_selection_handle = selection_handle.clone();
        let tag_list_delayed_all_selection = delayed_all_selection.clone();
        let tag_list_footer_handle = footer_handle.clone();
        tag_list_handle
            .borrow()
            .widget()
            .connect_row_selected(move |_list, row| {
                // do nothing if selection was cleared
                if row.is_none() {
                    return;
                }
                // deselect 'all' & tag_list
                Self::deselect_all_button(&tag_list_all_event_box, &tag_list_delayed_all_selection);
                tag_list_footer_handle.borrow().set_remove_button_sensitive(true);
                tag_list_feed_list_handle.borrow().deselect();

                if let Some(selected_id) = tag_list_tag_list_handle.borrow().get_selection() {
                    let selection = SidebarSelection::Tag(selected_id);
                    *tag_list_selection_handle.borrow_mut() = selection.clone();
                }
            });

        let scale = GtkUtil::get_scale(&sidebar);

        let expanded_categories = gtk_handle!(true);
        let expanded_tags = gtk_handle!(false);

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
        self.feed_list.borrow_mut().update(tree);
        self.sidebar.show_all();
    }

    pub fn update_taglist(&mut self, list: TagListModel) {
        self.tag_list.borrow_mut().update(list);
        self.sidebar.show_all();
    }

    pub fn hide_taglist(&self) {
        self.tags_box.hide();
    }

    pub fn show_taglist(&self) {
        self.tags_box.show_all();
        self.tag_list.borrow().widget().show_all();
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

    fn setup_expander(event_box: &EventBox, expander: &Image, revealer: &Revealer, expanded: &GtkHandle<bool>) {
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
                let is_expanded = *expanded.borrow();
                Self::expand_list(!is_expanded, &revealer, &expander, &expanded);
            }
            Inhibit(false)
        });
    }

    fn expand_list(expand: bool, revealer: &Revealer, expander: &Image, expanded: &GtkHandle<bool>) {
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
        *expanded.borrow_mut() = expand;
    }

    fn setup_all_button(
        event_box: &EventBox,
        sender: &Sender<Action>,
        feed_list_handle: GtkHandle<FeedList>,
        tag_list_handle: GtkHandle<TagList>,
        selection_handle: GtkHandle<SidebarSelection>,
        footer_handle: GtkHandle<SidebarFooter>,
        delayed_selection: &GtkHandle<Option<u32>>,
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

            feed_list_handle.borrow().deselect();
            tag_list_handle.borrow().deselect();

            Self::select_all_button(widget, &sender, &selection_handle, &delayed_selection);
            footer_handle.borrow().set_remove_button_sensitive(false);
            Inhibit(false)
        });
    }

    pub fn select_all_button_no_update(&self) {
        *self.selection.borrow_mut() = SidebarSelection::All;
        GtkUtil::remove_source(*self.delayed_all_selection.borrow());
        let context = self.all_event_box.get_style_context();
        context.add_class("selected");
    }

    fn select_all_button(
        all_event_box: &EventBox,
        sender: &Sender<Action>,
        selection_handle: &GtkHandle<SidebarSelection>,
        delayed_selection: &GtkHandle<Option<u32>>,
    ) {
        *selection_handle.borrow_mut() = SidebarSelection::All;
        let context = all_event_box.get_style_context();
        context.add_class("selected");

        GtkUtil::remove_source(*delayed_selection.borrow());
        let source_id = delayed_selection.clone();
        let sender = sender.clone();
        *delayed_selection.borrow_mut() = Some(
            gtk::timeout_add(300, move || {
                let sender = sender.clone();
                gtk::idle_add(move || {
                    GtkUtil::send(&sender, Action::SidebarSelection(SidebarSelection::All));
                    Continue(false)
                });

                *source_id.borrow_mut() = None;
                Continue(false)
            })
            .to_glib(),
        );
    }

    fn deselect_all_button(all_event_box: &EventBox, delayed_selection: &GtkHandle<Option<u32>>) {
        let context = all_event_box.get_style_context();
        context.remove_class("selected");
        GtkUtil::remove_source(*delayed_selection.borrow());
        *delayed_selection.borrow_mut() = None;
    }

    pub fn select_next_item(&self) -> Result<(), SidebarError> {
        let select_next = match *self.selection.borrow() {
            SidebarSelection::All => SidebarIterateItem::FeedListSelectFirstItem,
            SidebarSelection::Cateogry(_) | SidebarSelection::Feed(_) => self.feed_list.borrow().select_next_item(),
            SidebarSelection::Tag(_) => self.tag_list.borrow().get_next_item(),
        };
        self.select_item(select_next)
    }

    pub fn select_prev_item(&self) -> Result<(), SidebarError> {
        let select_next = match *self.selection.borrow() {
            SidebarSelection::All => SidebarIterateItem::TagListSelectLastItem,
            SidebarSelection::Cateogry(_) | SidebarSelection::Feed(_) => self.feed_list.borrow().select_prev_item(),
            SidebarSelection::Tag(_) => self.tag_list.borrow().get_prev_item(),
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
                    .borrow()
                    .set_selection(FeedListItemID::Feed(id))
                    .context(SidebarErrorKind::Selection)?;
            }
            SidebarIterateItem::SelectFeedListCategory(id) => {
                self.feed_list
                    .borrow()
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
                if let Some(item) = self.feed_list.borrow().get_first_item() {
                    self.feed_list
                        .borrow()
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
                if let Some(item) = self.feed_list.borrow().get_last_item(None) {
                    self.feed_list
                        .borrow()
                        .set_selection(item)
                        .context(SidebarErrorKind::Selection)?;
                }
            }
            SidebarIterateItem::SelectTagList(id) => {
                self.tag_list
                    .borrow()
                    .set_selection(id)
                    .context(SidebarErrorKind::Selection)?;
            }
            SidebarIterateItem::TagListSelectFirstItem => {
                // if tags not supported or not available jump back to "All Articles"
                if !self.tags_box.is_visible() {
                    return self.select_item(SidebarIterateItem::SelectAll);
                }
                Self::expand_list(true, &self.tags_revealer, &self.tags_expander, &self.expanded_tags);
                if let Some(item) = self.tag_list.borrow().get_first_item() {
                    self.tag_list
                        .borrow()
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
                if let Some(item) = self.tag_list.borrow().get_last_item() {
                    self.tag_list
                        .borrow()
                        .set_selection(item)
                        .context(SidebarErrorKind::Selection)?;
                }
            }
            SidebarIterateItem::NothingSelected => { /* nothing */ }
        }
        Ok(())
    }

    pub fn get_selection(&self) -> SidebarSelection {
        (*self.selection.borrow()).clone()
    }

    fn deselect(&self) {
        Self::deselect_all_button(&self.all_event_box, &self.delayed_all_selection);
        self.feed_list.borrow().cancel_selection();
        self.feed_list.borrow().widget().unselect_all();
        self.tag_list.borrow().cancel_selection();
        self.tag_list.borrow().widget().unselect_all();
    }

    pub fn expand_collapse_selected_category(&self) {
        self.feed_list.borrow().expand_collapse_selected_category()
    }

    pub fn get_add_button(&self) -> Button {
        self.footer.borrow().get_add_button()
    }
}
