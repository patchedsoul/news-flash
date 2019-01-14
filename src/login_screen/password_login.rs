use gtk::{
    self,
    ImageExt,
    WidgetExt,
    StyleContextExt,
    LabelExt,
};
use crate::gtk_util::GtkUtil;
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use news_flash::models::{
    PluginMetadata,
    PluginIcon,
};


#[derive(Clone, Debug)]
pub struct PasswordLogin {
    page: gtk::Box,
    logo: gtk::Image,
    headline: gtk::Label,
    scale_factor: i32,
}

impl PasswordLogin {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/password_login.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("password_login").ok_or(format_err!("some err"))?;
        let logo : gtk::Image = builder.get_object("logo").ok_or(format_err!("some err"))?;
        let headline : gtk::Label = builder.get_object("headline").ok_or(format_err!("some err"))?;

        let ctx = page.get_style_context().ok_or(format_err!("some err"))?;
        let scale = ctx.get_scale();

        let generic_logo_data = Resources::get("icons/feed_service_generic.svg").ok_or(format_err!("some err"))?;
        let surface = GtkUtil::create_surface_from_svg(&generic_logo_data, 64, 64, scale)?;
        logo.set_from_surface(&surface);

        let page = PasswordLogin {
            page: page,
            logo: logo,
            headline: headline,
            scale_factor: scale,
        };

        Ok(page)
    }

    pub fn set_service(&self, info: PluginMetadata) -> Result<(), Error> {
        
        // set Icon
        if let Some(icon) = info.icon {
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

        self.headline.set_text(&format!("Please log into {} and enjoy using NewsFlash", info.name));

        Ok(())
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }
}