use crate::color::ColorRGBA;
use crate::gtk_handle;
use crate::sidebar::tag_list::models::TagListTagModel;
use crate::util::GtkHandle;
use crate::Resources;
use cairo::{Context, FillRule};
use gdk::WindowExt;
use gtk::{self, ContainerExt, ImageExt, LabelExt, ListBoxRowExt, StyleContextExt, WidgetExt};
use news_flash::models::TagID;
use std::cell::RefCell;
use std::rc::Rc;
use std::str;

#[derive(Clone, Debug)]
pub struct TagRow {
    pub id: TagID,
    widget: gtk::ListBoxRow,
    item_count: gtk::Label,
    item_count_event: gtk::EventBox,
    title: gtk::Label,
    tag_color_circle: gtk::Image,
}

impl TagRow {
    pub fn new(model: &TagListTagModel) -> GtkHandle<Self> {
        let ui_data = Resources::get("ui/tag.ui").unwrap();
        let ui_string = str::from_utf8(ui_data.as_ref()).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let tag_box: gtk::Box = builder.get_object("tag_row").unwrap();
        let title_label: gtk::Label = builder.get_object("tag_title").unwrap();
        let item_count_label: gtk::Label = builder.get_object("item_count").unwrap();
        let item_count_event: gtk::EventBox = builder.get_object("item_count_event").unwrap();
        let tag_color_circle: gtk::Image = builder.get_object("tag_color").unwrap();

        let tag_image_update = tag_color_circle.clone();
        let tag_color_update = model.color.clone();
        tag_color_circle.connect_realize(move |_widget| {
            Self::update_color_cirlce_internal(&tag_image_update, &tag_color_update);
        });

        let tag = TagRow {
            id: model.id.clone(),
            widget: Self::create_row(&tag_box, &model.id),
            item_count: item_count_label,
            item_count_event,
            title: title_label,
            tag_color_circle,
        };
        tag.update_item_count(model.item_count);
        tag.update_title(&model.label);

        gtk_handle!(tag)
    }

    fn create_row(widget: &gtk::Box, _id: &TagID) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(false);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context();
        context.remove_class("activatable");

        row
    }

    pub fn row(&self) -> gtk::ListBoxRow {
        self.widget.clone()
    }

    pub fn update_item_count(&self, count: i32) {
        if count > 0 {
            self.item_count.set_label(&count.to_string());
            self.item_count_event.set_visible(true);
        } else {
            self.item_count_event.set_visible(false);
        }
    }

    // pub fn update_color_cirlce(&self, color: &str) {
    //     Self::update_color_cirlce_internal(&self.tag_color_circle, color);
    // }

    fn update_color_cirlce_internal(tag_color_circle: &gtk::Image, color: &str) {
        let size = 16;
        let half_size = f64::from(size / 2);
        let scale = tag_color_circle.get_style_context().get_scale();
        if let Some(window) = tag_color_circle.get_window() {
            if let Some(surface) = window.create_similar_image_surface(0, size, size, scale) {
                let cairo_ctx = Context::new(&surface);
                cairo_ctx.set_fill_rule(FillRule::EvenOdd);
                cairo_ctx.set_line_width(2.0);

                let rgba_outer = ColorRGBA::parse_string(color).unwrap();
                let mut rgba_inner = ColorRGBA::parse_string(color).unwrap();
                rgba_inner.adjust_lightness(0.05).unwrap();

                cairo_ctx.set_source_rgba(rgba_inner.red_normalized(), rgba_inner.green_normalized(), rgba_inner.blue_normalized(), rgba_inner.alpha_normalized());
                cairo_ctx.arc(half_size, half_size, half_size, 0.0, 2.0 * std::f64::consts::PI);
                cairo_ctx.fill_preserve();

                cairo_ctx.arc(half_size, half_size, half_size - (half_size / 4.0), 0.0, 2.0 * std::f64::consts::PI);
                cairo_ctx.set_source_rgba(rgba_outer.red_normalized(), rgba_outer.green_normalized(), rgba_outer.blue_normalized(), rgba_outer.alpha_normalized());
                cairo_ctx.fill_preserve();

                tag_color_circle.set_from_surface(&surface);
            }
        }
    }

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }
}
