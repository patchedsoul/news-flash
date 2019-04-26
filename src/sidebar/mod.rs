mod feed_list;
pub mod models;
mod tag_list;

use crate::gtk_handle;
use crate::util::GTK_RESOURCE_FILE_ERROR;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
use crate::Resources;
use failure::format_err;
use failure::Error;
pub use feed_list::models::{FeedListSelection, FeedListTree};
use feed_list::FeedList;
use gdk::{EventMask, EventType};
use gio::{ActionExt, ActionMapExt};
use glib::Variant;
use gtk::{
    Box, BoxExt, EventBox, Image, ImageExt, Label, LabelExt, ListBoxExt, Revealer, RevealerExt, StyleContextExt,
    WidgetExt, WidgetExtManual,
};
use models::SidebarSelection;
use news_flash::models::{PluginID, PluginIcon};
use news_flash::NewsFlash;
use std::cell::RefCell;
use std::rc::Rc;
pub use tag_list::models::TagListModel;
use tag_list::TagList;

#[derive(Clone, Debug)]
pub struct SideBar {
    sidebar: gtk::Box,
    logo: gtk::Image,
    unread_label: gtk::Label,
    service_label: gtk::Label,
    scale_factor: i32,
    feed_list: GtkHandle<FeedList>,
    tag_list: GtkHandle<TagList>,
}

impl SideBar {
    pub fn new() -> Result<Self, Error> {
        let builder = BuilderHelper::new("sidebar");

        let sidebar = builder.get::<Box>("toplevel");
        let logo = builder.get::<Image>("logo");
        let unread_label = builder.get::<Label>("unread_count_all");
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

        let feed_list = FeedList::new()?;
        let tag_list = TagList::new();

        let feed_list_handle = gtk_handle!(feed_list);
        let tag_list_handle = gtk_handle!(tag_list);

        feed_list_box.pack_start(&feed_list_handle.borrow().widget(), false, true, 0);
        tag_list_box.pack_start(&tag_list_handle.borrow().widget(), false, true, 0);

        let feed_list_all_event_box = all_event_box.clone();
        let feed_list_tag_list_handle = tag_list_handle.clone();
        let feed_list_feed_list_handle = feed_list_handle.clone();
        feed_list_handle
            .borrow()
            .widget()
            .connect_row_selected(move |list, row| {
                // do nothing if selection was cleared
                if row.is_none() {
                    return;
                }
                // deselect 'all' & tag_list
                let context = feed_list_all_event_box.get_style_context();
                context.remove_class("selected");
                feed_list_tag_list_handle.borrow().deselect();

                if let Some(selection) = feed_list_feed_list_handle.borrow().get_selection() {
                    let selection = SidebarSelection::from_feed_list_selection(selection);
                    if let Ok(selection_json) = serde_json::to_string(&selection) {
                        if let Ok(main_window) = GtkUtil::get_main_window(list) {
                            if let Some(action) = main_window.lookup_action("sidebar-selection") {
                                let selection = Variant::from(&selection_json);
                                action.activate(Some(&selection));
                            }
                        }
                    }
                }
            });

        let tag_list_all_event_box = all_event_box.clone();
        let tag_list_feed_list_handle = feed_list_handle.clone();
        let tag_list_tag_list_handle = tag_list_handle.clone();
        tag_list_handle
            .borrow()
            .widget()
            .connect_row_selected(move |list, row| {
                // do nothing if selection was cleared
                if row.is_none() {
                    return;
                }
                // deselect 'all' & tag_list
                let context = tag_list_all_event_box.get_style_context();
                context.remove_class("selected");
                tag_list_feed_list_handle.borrow().deselect();

                if let Some(selected_id) = tag_list_tag_list_handle.borrow().get_selection() {
                    let selection = SidebarSelection::Tag(selected_id);
                    if let Ok(selection_json) = serde_json::to_string(&selection) {
                        if let Ok(main_window) = GtkUtil::get_main_window(list) {
                            if let Some(action) = main_window.lookup_action("sidebar-selection") {
                                let selection = Variant::from(&selection_json);
                                action.activate(Some(&selection));
                            }
                        }
                    }
                }
            });

        let scale = sidebar.get_style_context().get_scale();

        let expanded_categories = gtk_handle!(true);
        let expanded_tags = gtk_handle!(false);

        Self::setup_expander(
            &categories_event_box,
            &categories_expander,
            &categories_revealer,
            &expanded_categories,
        );
        Self::setup_expander(&tags_event_box, &tags_expander, &tags_revealer, &expanded_tags);
        Self::setup_all_button(&all_event_box, feed_list_handle.clone(), tag_list_handle.clone());

        Ok(SideBar {
            sidebar,
            logo,
            unread_label,
            service_label,
            scale_factor: scale,
            feed_list: feed_list_handle,
            tag_list: tag_list_handle,
        })
    }

    pub fn widget(&self) -> gtk::Box {
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

    pub fn update_unread_all(&mut self, count: i64) {
        self.unread_label.set_text(&format!("{}", count));
    }

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), Error> {
        let list = NewsFlash::list_backends();
        let info = list.get(id).ok_or_else(|| format_err!("some err"))?;
        if let Some(icon) = &info.icon_symbolic {
            let surface = match icon {
                PluginIcon::Vector(icon) => {
                    GtkUtil::create_surface_from_bytes(&icon.data, icon.width, icon.height, self.scale_factor)?
                }
                PluginIcon::Pixel(icon) => GtkUtil::create_surface_from_pixelicon(icon, self.scale_factor)?,
            };
            self.logo.set_from_surface(&surface);
        } else {
            let generic_logo_data = Resources::get("icons/feed-service-generic.svg").expect(GTK_RESOURCE_FILE_ERROR);
            let surface = GtkUtil::create_surface_from_bytes(&generic_logo_data, 64, 64, self.scale_factor)?;
            self.logo.set_from_surface(&surface);
        }

        if let Some(user_name) = user_name {
            self.service_label.set_text(&user_name);
        } else {
            self.service_label.set_text(&info.name);
        }

        Ok(())
    }

    fn setup_expander(
        event_box: &gtk::EventBox,
        expander: &gtk::Image,
        revealer: &gtk::Revealer,
        expanded: &GtkHandle<bool>,
    ) {
        let expander = expander.clone();
        let expanded = expanded.clone();
        let revealer = revealer.clone();
        event_box.set_events(EventMask::BUTTON_PRESS_MASK);
        event_box.set_events(EventMask::ENTER_NOTIFY_MASK);
        event_box.set_events(EventMask::LEAVE_NOTIFY_MASK);
        event_box.connect_enter_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.add_class("highlight");
            gtk::Inhibit(false)
        });
        event_box.connect_leave_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.remove_class("highlight");
            gtk::Inhibit(false)
        });

        event_box.connect_button_press_event(move |_widget, event| {
            if event.get_event_type() == EventType::ButtonPress {
                if event.get_button() != 1 {
                    return gtk::Inhibit(false);
                }
                match event.get_event_type() {
                    EventType::ButtonPress => (),
                    _ => return gtk::Inhibit(false),
                }
                let context = expander.get_style_context();
                let expanded_bool = *expanded.borrow();

                if expanded_bool {
                    context.add_class("backward-arrow-collapsed");
                    context.remove_class("backward-arrow-expanded");
                    revealer.set_reveal_child(false);
                } else {
                    context.remove_class("backward-arrow-collapsed");
                    context.add_class("backward-arrow-expanded");
                    revealer.set_reveal_child(true);
                }

                *expanded.borrow_mut() = !expanded_bool;
            }
            gtk::Inhibit(false)
        });
    }

    fn setup_all_button(
        event_box: &gtk::EventBox,
        feed_list_handle: GtkHandle<FeedList>,
        tag_list_handle: GtkHandle<TagList>,
    ) {
        let context = event_box.get_style_context();
        context.add_class("selected");
        event_box.set_events(EventMask::BUTTON_PRESS_MASK);
        event_box.set_events(EventMask::ENTER_NOTIFY_MASK);
        event_box.set_events(EventMask::LEAVE_NOTIFY_MASK);
        event_box.connect_enter_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.add_class("highlight");
            gtk::Inhibit(false)
        });
        event_box.connect_leave_notify_event(|widget, _event| {
            let context = widget.get_style_context();
            context.remove_class("highlight");
            gtk::Inhibit(false)
        });

        event_box.connect_button_press_event(move |widget, event| {
            if event.get_button() != 1 {
                return gtk::Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }
            let context = widget.get_style_context();
            context.add_class("selected");
            feed_list_handle.borrow().deselect();
            tag_list_handle.borrow().deselect();

            let selection = SidebarSelection::All;
            if let Ok(selection_json) = serde_json::to_string(&selection) {
                if let Ok(main_window) = GtkUtil::get_main_window(widget) {
                    if let Some(action) = main_window.lookup_action("sidebar-selection") {
                        let selection = Variant::from(&selection_json);
                        action.activate(Some(&selection));
                    }
                }
            }
            gtk::Inhibit(false)
        });
    }
}
