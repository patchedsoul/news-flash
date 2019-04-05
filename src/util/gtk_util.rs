use gtk::{
    EntryExt,
    Cast,
    WidgetExt,
    ListBoxRow,
    BinExt,
    Revealer,
    StyleContext,
};
use gdk_pixbuf::{
    Pixbuf,
    Colorspace,
};
use gio::{
    MemoryInputStream,
};
use glib::{
    object::ObjectExt,
    signal::SignalHandlerId,
    source::SourceId,
    Bytes,
    object::IsA,
    translate::{
        FromGlib,
    },
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
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub type GtkHandle<T> = Rc<RefCell<T>>;
pub type GtkHandleMap<T, K> = GtkHandle<HashMap<T, K>>;

#[macro_export]
macro_rules! gtk_handle {
    ($x:expr) => {
        Rc::new(RefCell::new($x))
    }
}

pub struct GtkUtil;

impl GtkUtil {
    pub fn create_surface_from_pixelicon(icon: &PixelIcon, scale_factor: i32) -> Result<Surface, Error> {
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

    pub fn create_surface_from_bytes(data: &[u8], width: i32, height: i32, scale_factor: i32)-> Result<Surface, Error> {
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

    pub fn get_dnd_style_context_widget(row: &gtk::Widget) -> Option<StyleContext> {
        if let Ok(row) = row.clone().downcast::<ListBoxRow>() {
            return Self::get_dnd_style_context_listboxrow(&row)
        }

        None
    }

    pub fn get_dnd_style_context_listboxrow(row: &gtk::ListBoxRow) -> Option<StyleContext> {
        if let Some(row) = row.get_child() {
            if let Ok(row) = row.downcast::<Revealer>() {
                if let Some(row) = row.get_child() {
                    if let Some(style_context) = row.get_style_context() {
                        return Some(style_context)
                    }
                }
            }
        }

        None
    }

    pub fn disconnect_signal<T: ObjectExt>(signal_id: Option<u64>, widget: &T) {
        if let Some(signal_id) = signal_id {
            let signal_id = SignalHandlerId::from_glib(signal_id);
            widget.disconnect(signal_id);
        }
    }

    pub fn remove_source(source_id: Option<u32>) {
        if let Some(source_id) = source_id {
            let source_id = SourceId::from_glib(source_id);
            glib::source::source_remove(source_id);
        }
    }
}