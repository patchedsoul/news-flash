mod feed_list;
mod tag_list;
pub mod models;

use std::rc::Rc;
use std::cell::RefCell;
use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use gtk::{
    Builder,
    BoxExt,
    ImageExt,
    StyleContextExt,
    ListBoxExt,
    WidgetExt,
    LabelExt,
    RevealerExt,
};
use gdk::{
    EventMask,
    EventType,
};
use glib::Variant;
use gio::{
    ActionMapExt,
    ActionExt,
};
use crate::util::GtkUtil;
use news_flash::models::{
    PluginIcon,
    PluginID,
};
use news_flash::NewsFlash;
use models::SidebarSelection;
use crate::main_window::GtkHandle;
use feed_list::FeedList;
use tag_list::TagList;
pub use feed_list::models::{
    FeedListTree,
    FeedListSelection,
};
pub use tag_list::models::TagListModel;

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
        let ui_data = Resources::get("ui/sidebar.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let sidebar : gtk::Box = builder.get_object("toplevel").ok_or(format_err!("some err"))?;
        let logo : gtk::Image = builder.get_object("logo").ok_or(format_err!("some err"))?;
        let unread_label : gtk::Label = builder.get_object("unread_count_all").ok_or(format_err!("some err"))?;
        let service_label : gtk::Label = builder.get_object("service_label").ok_or(format_err!("some err"))?;
        let categories_event_box : gtk::EventBox = builder.get_object("categories_event_box").ok_or(format_err!("some err"))?;
        let categories_expander : gtk::Image = builder.get_object("categories_expander").ok_or(format_err!("some err"))?;
        let tags_event_box : gtk::EventBox = builder.get_object("tags_event_box").ok_or(format_err!("some err"))?;
        let tags_expander : gtk::Image = builder.get_object("tags_expander").ok_or(format_err!("some err"))?;
        let categories_revealer : gtk::Revealer = builder.get_object("categories_revealer").ok_or(format_err!("some err"))?;
        let tags_revealer : gtk::Revealer = builder.get_object("tags_revealer").ok_or(format_err!("some err"))?;
        let all_event_box : gtk::EventBox = builder.get_object("all_event_box").ok_or(format_err!("some err"))?;
        let feed_list_box : gtk::Box = builder.get_object("feed_list_box").ok_or(format_err!("some err"))?;
        let tag_list_box : gtk::Box = builder.get_object("tags_list_box").ok_or(format_err!("some err"))?;

        let feed_list = FeedList::new()?;
        let tag_list = TagList::new()?;
        
        let feed_list_handle = Rc::new(RefCell::new(feed_list));
        let tag_list_handle = Rc::new(RefCell::new(tag_list));

        feed_list_box.pack_start(&feed_list_handle.borrow().widget(), false, true, 0);
        tag_list_box.pack_start(&tag_list_handle.borrow().widget(), false, true, 0);

        let feed_list_all_event_box = all_event_box.clone();
        let feed_list_tag_list_handle = tag_list_handle.clone();
        let feed_list_feed_list_handle = feed_list_handle.clone();
        feed_list_handle.borrow().widget().connect_row_selected(move |list, row| {
            // do nothing if selection was cleared
            if row.is_none() {
                return
            }
            // deselect 'all' & tag_list
            let context = feed_list_all_event_box.get_style_context().unwrap();
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
        tag_list_handle.borrow().widget().connect_row_selected(move |list, row| {
            // do nothing if selection was cleared
            if row.is_none() {
                return
            }
            // deselect 'all' & tag_list
            let context = tag_list_all_event_box.get_style_context().unwrap();
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

        let scale = sidebar
            .get_style_context()
            .ok_or(format_err!("some err"))?
            .get_scale();

        let expanded_categories = Rc::new(RefCell::new(true));
        let expanded_tags =  Rc::new(RefCell::new(false));

        Self::setup_expander(&categories_event_box, &categories_expander, &categories_revealer, &expanded_categories);
        Self::setup_expander(&tags_event_box, &tags_expander, &tags_revealer, &expanded_tags);
        Self::setup_all_button(&all_event_box, feed_list_handle.clone(), tag_list_handle.clone());

        Ok(SideBar {
            sidebar: sidebar,
            logo: logo,
            unread_label: unread_label,
            service_label: service_label,
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
        let info = list.get(id).ok_or(format_err!("some err"))?;
        if let Some(icon) = &info.icon_symbolic {
            let surface = match icon {
                PluginIcon::Vector(icon) => {
                    GtkUtil::create_surface_from_bytes(&icon.data, icon.width, icon.height, self.scale_factor)?
                },
                PluginIcon::Pixel(icon) => {
                    GtkUtil::create_surface_from_pixelicon(icon, self.scale_factor)?
                },
            };
            self.logo.set_from_surface(&surface);
        }
        else {
            let generic_logo_data = Resources::get("icons/feed_service_generic.svg").ok_or(format_err!("some err"))?;
            let surface = GtkUtil::create_surface_from_bytes(&generic_logo_data, 64, 64, self.scale_factor)?;
            self.logo.set_from_surface(&surface);
        }

        if let Some(user_name) = user_name {
            self.service_label.set_text(&user_name);
        }
        else {
            self.service_label.set_text(&info.name);
        }

        Ok(())
    }

    fn setup_expander(
        event_box: &gtk::EventBox,
        expander: &gtk::Image,
        revealer: &gtk::Revealer,
        expanded: &GtkHandle<bool>
    ) {
        let expander = expander.clone();
        let expanded = expanded.clone();
        let revealer = revealer.clone();
        event_box.set_events(EventMask::BUTTON_PRESS_MASK.bits() as i32);
        event_box.set_events(EventMask::ENTER_NOTIFY_MASK.bits() as i32);
        event_box.set_events(EventMask::LEAVE_NOTIFY_MASK.bits() as i32);
        event_box.connect_enter_notify_event(|widget, _event| {
            let context = widget.get_style_context().unwrap();
            context.add_class("highlight");
            gtk::Inhibit(false)
        });
        event_box.connect_leave_notify_event(|widget, _event| {
            let context = widget.get_style_context().unwrap();
            context.remove_class("highlight");
            gtk::Inhibit(false)
        });

        event_box.connect_button_press_event(move |_widget, event| {
            if event.get_event_type() == EventType::ButtonPress {
                if event.get_button() != 1 {
                    return gtk::Inhibit(false)
                }
                match event.get_event_type() {
                    EventType::ButtonPress => (),
                    _ => return gtk::Inhibit(false),
                }
                let context = expander.get_style_context().unwrap();
                let expanded_bool = *expanded.borrow();

                if expanded_bool {
                    context.add_class("backward-arrow-collapsed");
                    context.remove_class("backward-arrow-expanded");
                    revealer.set_reveal_child(false);
                }
                else {
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
        tag_list_handle: GtkHandle<TagList>
    ) {
        let context = event_box.get_style_context().unwrap();
        context.add_class("selected");
        event_box.set_events(EventMask::BUTTON_PRESS_MASK.bits() as i32);
        event_box.set_events(EventMask::ENTER_NOTIFY_MASK.bits() as i32);
        event_box.set_events(EventMask::LEAVE_NOTIFY_MASK.bits() as i32);
        event_box.connect_enter_notify_event(|widget, _event| {
            let context = widget.get_style_context().unwrap();
            context.add_class("highlight");
            gtk::Inhibit(false)
        });
        event_box.connect_leave_notify_event(|widget, _event| {
            let context = widget.get_style_context().unwrap();
            context.remove_class("highlight");
            gtk::Inhibit(false)
        });

        event_box.connect_button_press_event(move |widget, event| {
            if event.get_button() != 1 {
                return gtk::Inhibit(false)
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }
            let context = widget.get_style_context().unwrap();
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