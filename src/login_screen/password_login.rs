use gtk::{
    self,
    ImageExt,
    WidgetExt,
    StyleContextExt,
    LabelExt,
};
use gdk_pixbuf::{
    Pixbuf,
    Colorspace,
};
use gio::{
    MemoryInputStream,
};
use glib::{
    Bytes,
};
use cairo::{
    Context,
};
use gdk::ContextExt;
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
        let generic_logo_bytes = Bytes::from(&generic_logo_data);
        let stream = MemoryInputStream::new_from_bytes(&generic_logo_bytes);
        let pixbuf = Pixbuf::new_from_stream_at_scale(
            &stream,
            64 * scale,
            64 * scale,
            true,
            None
        )?;
        let surface = Context::cairo_surface_create_from_pixbuf(&pixbuf, scale, None);
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
            let pixbuf = match icon {
                PluginIcon::Vector(icon) => {
                    let bytes = Bytes::from(&icon.data);
                    let stream = MemoryInputStream::new_from_bytes(&bytes);
                    Pixbuf::new_from_stream_at_scale(
                        &stream,
                        64 * self.scale_factor,
                        64 * self.scale_factor,
                        true,
                        None
                    )?
                },
                PluginIcon::Pixel(icon) => {
                    Pixbuf::new_from_vec(
                        icon.data, 
                        Colorspace::Rgb,
                        icon.has_alpha, 
                        icon.bits_per_sample, 
                        icon.width, 
                        icon.height, 
                        icon.row_stride,
                    )
                },
            };
            let surface = Context::cairo_surface_create_from_pixbuf(&pixbuf, self.scale_factor, None);
            self.logo.set_from_surface(&surface);
        }

        self.headline.set_text(&format!("Please log into {} and enjoy using NewsFlash", info.name));

        Ok(())
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }
}