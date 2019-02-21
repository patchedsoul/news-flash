use gtk::{
    EntryExt,
    Cast,
    WidgetExt,
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
    object::IsA,
};
use cairo::{
    Context,
};
use news_flash::models::{
    PixelIcon,
};
use cairo::{
    Surface,
};
use gdk::{
    ContextExt,
};
use failure::{
    Error,
    format_err,
};

pub struct GtkUtil;

impl GtkUtil {
    pub fn create_surface_from_bitmap(icon: &PixelIcon, scale_factor: i32) -> Result<Surface, Error> {
        let pixbuf = Pixbuf::new_from_vec(
            icon.data.clone(),
            Colorspace::Rgb,
            icon.has_alpha,
            icon.bits_per_sample,
            icon.width,
            icon.height,
            icon.row_stride,
        );
        Context::cairo_surface_create_from_pixbuf(&pixbuf, scale_factor, None)
            .ok_or(format_err!("some err"))
    }

    pub fn create_surface_from_svg(data: &[u8], width: i32, height: i32, scale_factor: i32)-> Result<Surface, Error> {
        let bytes = Bytes::from(data);
        let stream = MemoryInputStream::new_from_bytes(&bytes);
        let pixbuf = Pixbuf::new_from_stream_at_scale(
            &stream,
            width * scale_factor,
            height * scale_factor,
            true,
            None
        )?;
        Context::cairo_surface_create_from_pixbuf(&pixbuf, scale_factor, None)
            .ok_or(format_err!("some err"))
    }

    pub fn is_entry_emty(entry: &gtk::Entry) -> bool {
        if entry.get_text_length() == 0 {
            return true;
        }
        false
    }

    pub fn is_main_window<W: IsA<gtk::Object> + IsA<gtk::Widget> + Clone>(widget: &W) -> bool {
        widget.clone().upcast::<gtk::Widget>().is::<gtk::ApplicationWindow>()
    }

    pub fn get_main_window<W: IsA<gtk::Object> + IsA<gtk::Widget> + WidgetExt + Clone>(widget: &W) -> Result<gtk::ApplicationWindow, Error> {
        if let Some(toplevel) = widget.get_toplevel() {
            if Self::is_main_window(&toplevel) {
                let main_window = toplevel.downcast::<gtk::ApplicationWindow>().unwrap();
                return Ok(main_window)
            }
        }
        
        Err(format_err!("some err"))
    }
}