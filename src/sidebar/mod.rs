//mod feed_list;

//pub use crate::sidebar::feed_list::category_row::CategoryRow;
//pub use crate::sidebar::feed_list::feed_row::FeedRow;
//pub use crate::sidebar::feed_list::feed_list::FeedList;

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
};
use crate::gtk_util::GtkUtil;
use news_flash::models::{
    PluginIcon,
    PluginID,
};
use news_flash::NewsFlash;

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

        let scale = sidebar
            .get_style_context()
            .ok_or(format_err!("some err"))?
            .get_scale();


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

    pub fn set_service(&self, id: &PluginID) -> Result<(), Error> {
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
        
        self.service_label.set_text(&info.name);

        Ok(())
    }
}