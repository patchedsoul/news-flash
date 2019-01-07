use gtk::{
    self,
    LabelExt,
    ImageExt,
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
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use news_flash::models::{
    PluginMetadata,
    PluginIcon,
};

#[derive(Clone, Debug)]
pub struct ServiceRow {
    pub(crate) widget: gtk::EventBox,
}

impl ServiceRow {
    pub fn new(info: PluginMetadata) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/service_row.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let row : gtk::EventBox = builder.get_object("service_row").ok_or(format_err!("some err"))?;
        let label : gtk::Label = builder.get_object("label").ok_or(format_err!("some err"))?;
        label.set_label(&info.name);

        let image : gtk::Image = builder.get_object("icon").ok_or(format_err!("get icon widget"))?;

        if let Some(icon) = info.icon {

            match icon {
                PluginIcon::Vector(icon) => {
                    let bytes = Bytes::from(&icon.data);
                    let stream = MemoryInputStream::new_from_bytes(&bytes);
                    let pixbuf = Pixbuf::new_from_stream(&stream, None)?;
                    image.set_from_pixbuf(&pixbuf);
                },
                PluginIcon::Pixel(icon) => {
                    let pixbuf = Pixbuf::new_from_vec(
                        icon.data, 
                        Colorspace::Rgb,
                        icon.has_alpha, 
                        icon.bits_per_sample, 
                        icon.width, 
                        icon.height, 
                        icon.row_stride,
                    );
                    image.set_from_pixbuf(&pixbuf);
                },
            }
            
            
        }
        else {
            // FIXME: default Icon
        }

        Ok(ServiceRow {
            widget: row.clone(),
        })
    }
}