//mod feed_list;

//pub use crate::sidebar::feed_list::category_row::CategoryRow;
//pub use crate::sidebar::feed_list::feed_row::FeedRow;
//pub use crate::sidebar::feed_list::feed_list::FeedList;

use std::rc::Rc;
use std::cell::RefCell;
use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use gtk::{
    Builder,
    ImageExt,
    StyleContextExt,
    WidgetExt,
    LabelExt,
    RevealerExt,
};
use gdk::{
    EventMask,
    EventType,
};
use crate::gtk_util::GtkUtil;
use news_flash::models::{
    PluginIcon,
    PluginID,
};
use news_flash::NewsFlash;
use crate::main_window::GtkHandle;

#[derive(Clone, Debug)]
pub struct SideBar {
    sidebar: gtk::Box,
    logo: gtk::Image,
    service_label: gtk::Label,
    scale_factor: i32,
}

impl SideBar {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/sidebar.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let sidebar : gtk::Box = builder.get_object("toplevel").ok_or(format_err!("some err"))?;
        let logo : gtk::Image = builder.get_object("logo").ok_or(format_err!("some err"))?;
        let service_label : gtk::Label = builder.get_object("service_label").ok_or(format_err!("some err"))?;
        let all_icon : gtk::Image = builder.get_object("all_icon").ok_or(format_err!("some err"))?;
        let categories_event_box : gtk::EventBox = builder.get_object("categories_event_box").ok_or(format_err!("some err"))?;
        let categories_expander : gtk::Image = builder.get_object("categories_expander").ok_or(format_err!("some err"))?;
        let tags_event_box : gtk::EventBox = builder.get_object("tags_event_box").ok_or(format_err!("some err"))?;
        let tags_expander : gtk::Image = builder.get_object("tags_expander").ok_or(format_err!("some err"))?;
        let categories_revealer : gtk::Revealer = builder.get_object("categories_revealer").ok_or(format_err!("some err"))?;
        let tags_revealer : gtk::Revealer = builder.get_object("tags_revealer").ok_or(format_err!("some err"))?;
        let all_event_box : gtk::EventBox = builder.get_object("all_event_box").ok_or(format_err!("some err"))?;

        let scale = sidebar
            .get_style_context()
            .ok_or(format_err!("some err"))?
            .get_scale();

        let expanded_categories = Rc::new(RefCell::new(true));
        let expanded_tags =  Rc::new(RefCell::new(true));

        Self::setup_expander(&categories_event_box, &categories_expander, &categories_revealer, &expanded_categories);
        Self::setup_expander(&tags_event_box, &tags_expander, &tags_revealer, &expanded_tags);
        Self::setup_all_button(&all_event_box);

        let icon = Resources::get("icons/feed_service_generic_symbolic.svg").ok_or(format_err!("some err"))?;
        let icon = GtkUtil::create_surface_from_svg(&icon, 16, 16, scale)?;
        all_icon.set_from_surface(&icon);

        Ok(SideBar {
            sidebar: sidebar,
            logo: logo,
            service_label: service_label,
            scale_factor: scale,
        })
    }

    pub fn widget(&self) -> gtk::Box {
        self.sidebar.clone()
    }

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), Error> {
        let list = NewsFlash::list_backends();
        let info = list.get(id).ok_or(format_err!("some err"))?;
        if let Some(icon) = &info.icon_symbolic {
            let surface = match icon {
                PluginIcon::Vector(icon) => {
                    GtkUtil::create_surface_from_svg(&icon.data, icon.width, icon.height, self.scale_factor)?
                },
                PluginIcon::Pixel(icon) => {
                    GtkUtil::create_surface_from_bitmap(icon, self.scale_factor)?
                },
            };
            self.logo.set_from_surface(&surface);
        }
        else {
            let generic_logo_data = Resources::get("icons/feed_service_generic.svg").ok_or(format_err!("some err"))?;
            let surface = GtkUtil::create_surface_from_svg(&generic_logo_data, 64, 64, self.scale_factor)?;
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
                let context = expander.get_style_context().unwrap();
                let expanded_bool = *expanded.borrow();

                if expanded_bool {
                    context.add_class("forward-arrow-collapsed");
                    context.remove_class("forward-arrow-expanded");
                    revealer.set_reveal_child(false);
                }
                else {
                    context.remove_class("forward-arrow-collapsed");
                    context.add_class("forward-arrow-expanded");
                    revealer.set_reveal_child(true);
                }

                *expanded.borrow_mut() = !expanded_bool;
            }
            gtk::Inhibit(false)
        });
    }

    fn setup_all_button(event_box: &gtk::EventBox) {
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

        event_box.connect_button_press_event(move |_widget, _event| {

            gtk::Inhibit(false)
        });
    }
}