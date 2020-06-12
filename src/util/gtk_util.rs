use super::constants;
use super::error::{UtilError, UtilErrorKind};
use crate::color::ColorRGBA;
use crate::Resources;
use cairo::{Context, FillRule, ImageSurface, Surface};
use failure::ResultExt;
use gdk::{prelude::GdkContextExt, Window, WindowExt};
use gdk_pixbuf::Pixbuf;
use gio::{Cancellable, MemoryInputStream, Resource};
use glib::{
    object::{Cast, IsA, Object, ObjectExt},
    signal::SignalHandlerId,
    source::SourceId,
    translate::FromGlib,
    Bytes,
};
use gtk::{
    BinExt, EntryExt, EventBox, IconTheme, IconThemeExt, ListBoxRow, Revealer, StyleContext, StyleContextExt, WidgetExt,
};
use log::{error, warn};
use news_flash::models::PixelIcon;

pub const GTK_RESOURCE_FILE_ERROR: &str = "Could not load file from resources. This should never happen!";
pub const GTK_BUILDER_ERROR: &str = "Could not build GTK widget from UI file. This should never happen!";
pub const GTK_CSS_ERROR: &str = "Could not load CSS. This should never happen!";

pub struct GtkUtil;

impl GtkUtil {
    pub fn register_symbolic_icons() {
        let data = Resources::get("gresource_bundles/symbolic_icons.gresource").expect(GTK_RESOURCE_FILE_ERROR);
        let bytes = Bytes::from(&data);
        let icon_resource = Resource::new_from_data(&bytes).expect("Error creating gio resource.");
        gio::resources_register(&icon_resource);
        IconTheme::get_default()
            .unwrap_or_else(|| panic!("Failed to register symbolic icons."))
            .add_resource_path("/com/gitlab/newsflash/resources/icons/");
    }

    pub fn create_surface_from_icon_name(icon_name: &str, size: i32, scale_factor: i32) -> Surface {
        let icon_name = format!("icons/{}.svg", icon_name);
        let icon = Resources::get(&icon_name).expect(GTK_RESOURCE_FILE_ERROR);
        Self::create_surface_from_bytes(&icon, size, size, scale_factor)
            .unwrap_or_else(|_| panic!("Failed to load '{}' from resources.", icon_name))
    }

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
        let surface = match ImageSurface::create(cairo::Format::ARgb32, width * scale_factor, height * scale_factor) {
            Ok(surface) => surface,
            Err(_) => return Err(UtilErrorKind::CairoSurface)?,
        };
        let ctx = Context::new(&surface);
        ctx.set_source_pixbuf(&pixbuf, 0.0, 0.0);
        ctx.paint();
        let surface = ctx.get_target();
        surface.set_device_scale(scale_factor as f64, scale_factor as f64);
        Ok(surface)
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

    pub fn get_scale<W: IsA<Object> + IsA<gtk::Widget> + Clone>(widget: &W) -> i32 {
        widget.get_style_context().get_scale()
    }

    pub fn is_main_window<W: IsA<Object> + IsA<gtk::Widget> + Clone>(widget: &W) -> bool {
        widget.clone().upcast::<gtk::Widget>().is::<gtk::ApplicationWindow>()
    }

    pub fn get_main_window<W: IsA<Object> + IsA<gtk::Widget> + WidgetExt + Clone>(
        widget: &W,
    ) -> Result<gtk::ApplicationWindow, UtilError> {
        if let Some(toplevel) = widget.get_toplevel() {
            if Self::is_main_window(&toplevel) {
                let main_window = toplevel
                    .downcast::<gtk::ApplicationWindow>()
                    .expect("Already checked if toplevel is main_window");
                return Ok(main_window);
            }
            warn!("widget is not the main window");
        } else {
            warn!("widget is not a toplevel");
        }

        error!("getting main window for widget failed");
        Err(UtilErrorKind::WidgetIsMainwindow.into())
    }

    pub fn get_dnd_style_context_widget(row: &gtk::Widget) -> Option<StyleContext> {
        if let Ok(row) = row.clone().downcast::<ListBoxRow>() {
            return Self::get_dnd_style_context_listboxrow(&row);
        }

        warn!("Couldn't cast widget to ListBoxRow");
        None
    }

    pub fn listboxrow_is_category(row: &gtk::ListBoxRow) -> bool {
        if let Some(row) = row.get_child() {
            if row.downcast::<EventBox>().is_ok() {
                return true;
            }
        }

        false
    }

    pub fn get_dnd_style_context_listboxrow(row: &gtk::ListBoxRow) -> Option<StyleContext> {
        if let Some(row) = row.get_child() {
            if let Ok(row) = row.clone().downcast::<Revealer>() {
                if let Some(row) = row.get_child() {
                    return Some(row.get_style_context());
                }
            } else if let Ok(row) = row.downcast::<EventBox>() {
                if let Some(row) = row.get_child() {
                    if let Ok(row) = row.downcast::<Revealer>() {
                        if let Some(row) = row.get_child() {
                            return Some(row.get_style_context());
                        }
                    }
                }
            }
        }

        None
    }

    pub fn disconnect_signal<T: ObjectExt>(signal_id: Option<usize>, widget: &T) {
        if let Some(signal_id) = signal_id {
            #[cfg(target_pointer_width = "64")]
            let signal_id = SignalHandlerId::from_glib(signal_id as u64);
            #[cfg(target_pointer_width = "32")]
            let signal_id = SignalHandlerId::from_glib(signal_id as u32);
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

    pub fn generate_color_cirlce(window: &Window, color: Option<&str>, scale: i32) -> Option<cairo::Surface> {
        let size = 16;
        let half_size = f64::from(size / 2);

        if let Some(surface) = window.create_similar_image_surface(0, size * scale, size * scale, scale) {
            let cairo_ctx = Context::new(&surface);
            cairo_ctx.set_fill_rule(FillRule::EvenOdd);
            cairo_ctx.set_line_width(2.0);

            let color = match color {
                Some(color) => color,
                None => constants::TAG_DEFAULT_OUTER_COLOR,
            };
            let rgba_outer = match ColorRGBA::parse_string(color) {
                Ok(color) => color,
                Err(_) => ColorRGBA::parse_string(constants::TAG_DEFAULT_OUTER_COLOR)
                    .expect("Failed to parse default outer RGBA string."),
            };

            let mut rgba_inner = rgba_outer;
            if rgba_inner.adjust_lightness(0.05).is_err() {
                rgba_inner = ColorRGBA::parse_string(constants::TAG_DEFAULT_INNER_COLOR)
                    .expect("Failed to parse default inner RGBA string.")
            }

            cairo_ctx.set_source_rgba(
                rgba_inner.red_normalized(),
                rgba_inner.green_normalized(),
                rgba_inner.blue_normalized(),
                rgba_inner.alpha_normalized(),
            );
            cairo_ctx.arc(half_size, half_size, half_size, 0.0, 2.0 * std::f64::consts::PI);
            cairo_ctx.fill_preserve();

            cairo_ctx.arc(
                half_size,
                half_size,
                half_size - (half_size / 4.0),
                0.0,
                2.0 * std::f64::consts::PI,
            );
            cairo_ctx.set_source_rgba(
                rgba_outer.red_normalized(),
                rgba_outer.green_normalized(),
                rgba_outer.blue_normalized(),
                rgba_outer.alpha_normalized(),
            );
            cairo_ctx.fill_preserve();
            return Some(surface);
        }
        None
    }
}
