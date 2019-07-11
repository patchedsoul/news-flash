use super::error::{UtilError, UtilErrorKind};
use cairo::{Context, Surface};
use failure::ResultExt;
use gdk::{ContextExt, Window};
use gdk_pixbuf::Pixbuf;
use gio::{Cancellable, MemoryInputStream};
use glib::{object::IsA, object::ObjectExt, signal::SignalHandlerId, source::SourceId, translate::FromGlib, Bytes};
use gtk::{BinExt, Cast, EntryExt, ListBoxRow, Revealer, StyleContext, WidgetExt};
use log::{error, warn};
use news_flash::models::PixelIcon;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type GtkHandle<T> = Rc<RefCell<T>>;
pub type GtkHandleMap<T, K> = GtkHandle<HashMap<T, K>>;

pub const GTK_RESOURCE_FILE_ERROR: &str = "Could not load file from resources. This should never happen!";
pub const GTK_BUILDER_ERROR: &str = "Could not build GTK widget from UI file. This should never happen!";
pub const GTK_CSS_ERROR: &str = "Could not load CSS. This should never happen!";

#[macro_export]
macro_rules! gtk_handle {
    ($x:expr) => {
        Rc::new(RefCell::new($x))
    };
}

pub struct GtkUtil;

impl GtkUtil {
    pub fn create_surface_from_pixelicon(icon: &PixelIcon, scale_factor: i32) -> Result<Surface, UtilError> {
        Self::create_surface_from_bytes(&icon.data, icon.width, icon.height, scale_factor)
    }

    pub fn create_surface_from_bytes(
        data: &[u8],
        width: i32,
        height: i32,
        scale_factor: i32,
    ) -> Result<Surface, UtilError> {
        let pixbuf = Self::create_pixbuf_from_bytes(data, width, height, scale_factor)?;
        let window : Option<&Window> = None;
        match Context::cairo_surface_create_from_pixbuf(&pixbuf, scale_factor, window) {
            Some(surface) => return Ok(surface),
            None => return Err(UtilErrorKind::CairoSurface)?,
        }
    }

    pub fn create_pixbuf_from_bytes(
        data: &[u8],
        width: i32,
        height: i32,
        scale_factor: i32,
    ) -> Result<Pixbuf, UtilError> {
        let bytes = Bytes::from(data);
        let stream = MemoryInputStream::new_from_bytes(&bytes);
        let cancellable: Option<&Cancellable> = None;
        let pixbuf =
            Pixbuf::new_from_stream_at_scale(&stream, width * scale_factor, height * scale_factor, true, cancellable)
                .context(UtilErrorKind::CairoSurface)?;
        Ok(pixbuf)
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

    pub fn get_main_window<W: IsA<gtk::Object> + IsA<gtk::Widget> + WidgetExt + Clone>(
        widget: &W,
    ) -> Result<gtk::ApplicationWindow, UtilError> {
        if let Some(toplevel) = widget.get_toplevel() {
            if Self::is_main_window(&toplevel) {
                let main_window = toplevel.downcast::<gtk::ApplicationWindow>().unwrap();
                return Ok(main_window);
            }
            warn!("widget is not the main window");
        } else {
            warn!("widget is not a toplevel");
        }

        error!("getting main window for widget failed");
        Err(UtilErrorKind::WidgetIsMainwindow)?
    }

    pub fn get_dnd_style_context_widget(row: &gtk::Widget) -> Option<StyleContext> {
        if let Ok(row) = row.clone().downcast::<ListBoxRow>() {
            return Self::get_dnd_style_context_listboxrow(&row);
        }

        warn!("Couldn't cast widget to ListBoxRow");
        None
    }

    pub fn get_dnd_style_context_listboxrow(row: &gtk::ListBoxRow) -> Option<StyleContext> {
        if let Some(row) = row.get_child() {
            if let Ok(row) = row.downcast::<Revealer>() {
                if let Some(row) = row.get_child() {
                    return Some(row.get_style_context());
                }
            }
        }

        None
    }

    pub fn disconnect_signal_handle<T: ObjectExt>(signal_id: &GtkHandle<Option<u64>>, widget: &T) {
        Self::disconnect_signal(*signal_id.borrow(), widget);
    }

    pub fn disconnect_signal<T: ObjectExt>(signal_id: Option<u64>, widget: &T) {
        if let Some(signal_id) = signal_id {
            let signal_id = SignalHandlerId::from_glib(signal_id);
            widget.disconnect(signal_id);
        }
        //warn!("Signal ID to disconnect is NONE");
    }

    pub fn remove_source(source_id: Option<u32>) {
        if let Some(source_id) = source_id {
            let source_id = SourceId::from_glib(source_id);
            glib::source::source_remove(source_id);
        }
        //warn!("Source ID to remove is NONE");
    }
}
