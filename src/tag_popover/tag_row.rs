
use crate::util::{BuilderHelper, constants, GtkUtil};
use glib::clone;
use gtk::{Box, ContainerExt, EventBox, Image, ImageExt, Label, LabelExt, ListBoxRow, ListBoxRowExt, StyleContextExt, WidgetExt};
use news_flash::models::{Tag, TagID};

#[derive(Clone, Debug)]
pub struct TagRow {
    pub id: TagID,
    pub widget: ListBoxRow,
}

impl TagRow {
    pub fn new(tag: &Tag, assigned: bool) -> Self {
        let builder = BuilderHelper::new("tag");
        let tag_box = builder.get::<Box>("tag_row");
        let title_label = builder.get::<Label>("tag_title");
        let tag_color_circle = builder.get::<Image>("tag_color");
        let remove_event = builder.get::<EventBox>("remove_event");

        remove_event.set_no_show_all(!assigned);

        tag_color_circle.connect_realize(
            clone!(@weak tag_color_circle, @strong tag.color as color => move |_widget| {
                if let Some(window) = tag_color_circle.get_window() {
                    let scale = GtkUtil::get_scale(&tag_color_circle);
                    let color = match &color {
                        Some(color) => color,
                        None => constants::TAG_DEFAULT_INNER_COLOR,
                    };
                    if let Some(surface) = GtkUtil::generate_color_cirlce(&window, &color, scale) {
                        tag_color_circle.set_from_surface(Some(&surface));
                    }
                }
            }),
        );

        let tag_row = TagRow {
            id: tag.tag_id.clone(),
            widget: Self::create_row(&tag_box),
        };
        title_label.set_label(&tag.label);

        tag_row
    }

    fn create_row(widget: &Box) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context();
        context.remove_class("activatable");

        row
    }
}
