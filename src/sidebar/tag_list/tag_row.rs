use crate::sidebar::tag_list::models::TagListTagModel;
use crate::util::{BuilderHelper, GtkUtil};
use glib::clone;
use gtk::{Box, ContainerExt, Image, ImageExt, Label, LabelExt, ListBoxRow, ListBoxRowExt, StyleContextExt, WidgetExt};
use news_flash::models::TagID;
use parking_lot::RwLock;
use std::str;
use std::sync::Arc;

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

        tag_color_circle.connect_realize(
            clone!(@weak tag_color_circle, @strong model.color as color => move |_widget| {
                if let Some(window) = tag_color_circle.get_window() {
                    let scale = GtkUtil::get_scale(&tag_color_circle);
                    if let Some(surface) = GtkUtil::generate_color_cirlce(&window, &color, scale) {
                        tag_color_circle.set_from_surface(Some(&surface));
                    }
                }
            }),
        );

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

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }
}
