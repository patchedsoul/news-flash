use crate::color::ColorRGBA;
use crate::sidebar::tag_list::models::TagListTagModel;
use crate::util::{BuilderHelper, GtkUtil};
use cairo::{Context, FillRule};
use gdk::WindowExt;
use gtk::{Box, ContainerExt, Image, ImageExt, Label, LabelExt, ListBoxRow, ListBoxRowExt, StyleContextExt, WidgetExt};
use news_flash::models::TagID;
use std::sync::Arc;
use parking_lot::RwLock;
use std::str;

const DEFAULT_OUTER_COLOR: &str = "#FF0077";
const DEFAULT_INNER_COLOR: &str = "#FF0077";

#[derive(Clone, Debug)]
pub struct TagRow {
    pub id: TagID,
    widget: ListBoxRow,
    title: Label,
    tag_color_circle: Image,
}

impl TagRow {
    pub fn new(model: &TagListTagModel) -> Arc<RwLock<Self>> {
        let builder = BuilderHelper::new("tag");
        let tag_box = builder.get::<Box>("tag_row");
        let title_label = builder.get::<Label>("tag_title");
        let tag_color_circle = builder.get::<Image>("tag_color");

        let tag_image_update = tag_color_circle.clone();
        let tag_color_update = model.color.clone();
        tag_color_circle.connect_realize(move |_widget| {
            Self::update_color_cirlce(&tag_image_update, &tag_color_update);
        });

        let tag = TagRow {
            id: model.id.clone(),
            widget: Self::create_row(&tag_box, &model.id),
            title: title_label,
            tag_color_circle,
        };
        tag.update_title(&model.label);

        Arc::new(RwLock::new(tag))
    }

    fn create_row(widget: &Box, _id: &TagID) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context();
        context.remove_class("activatable");

        row
    }

    pub fn widget(&self) -> ListBoxRow {
        self.widget.clone()
    }

    fn update_color_cirlce(tag_color_circle: &Image, color: &str) {
        let size = 16;
        let half_size = f64::from(size / 2);
        let scale = GtkUtil::get_scale(tag_color_circle);
        if let Some(window) = tag_color_circle.get_window() {
            if let Some(surface) = window.create_similar_image_surface(0, size, size, scale) {
                let cairo_ctx = Context::new(&surface);
                cairo_ctx.set_fill_rule(FillRule::EvenOdd);
                cairo_ctx.set_line_width(2.0);

                let rgba_outer = match ColorRGBA::parse_string(color) {
                    Ok(color) => color,
                    Err(_) => ColorRGBA::parse_string(DEFAULT_OUTER_COLOR)
                        .expect("Failed to parse default outer RGBA string."),
                };

                let mut rgba_inner = rgba_outer;
                if rgba_inner.adjust_lightness(0.05).is_err() {
                    rgba_inner = ColorRGBA::parse_string(DEFAULT_INNER_COLOR)
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

                tag_color_circle.set_from_surface(Some(&surface));
            }
        }
    }

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }
}
