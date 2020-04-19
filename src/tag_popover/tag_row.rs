use crate::util::{constants, BuilderHelper, GtkUtil};
use gdk::{EventMask, NotifyType};
use glib::clone;
use gtk::{
    prelude::WidgetExtManual, Box, ContainerExt, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, ListBoxRow,
    ListBoxRowExt, StyleContextExt, WidgetExt,
};
use news_flash::models::{Tag, TagID};

#[derive(Clone, Debug)]
pub struct TagRow {
    pub id: TagID,
    pub widget: ListBoxRow,
    pub eventbox: EventBox,
}

impl TagRow {
    pub fn new(tag: &Tag, assigned: bool) -> Self {
        let builder = BuilderHelper::new("tag");
        let tag_box = builder.get::<Box>("tag_row");
        let title_label = builder.get::<Label>("tag_title");
        let tag_color_circle = builder.get::<Image>("tag_color");
        let remove_event = builder.get::<EventBox>("remove_event");

        remove_event.set_no_show_all(!assigned);
        remove_event.set_events(EventMask::BUTTON_PRESS_MASK);
        remove_event.set_events(EventMask::ENTER_NOTIFY_MASK);
        remove_event.set_events(EventMask::LEAVE_NOTIFY_MASK);
        remove_event.connect_enter_notify_event(|widget, event| {
            if event.get_detail() != NotifyType::Inferior {
                widget.set_opacity(1.0);
            }
            Inhibit(false)
        });
        remove_event.connect_leave_notify_event(|widget, event| {
            if event.get_detail() != NotifyType::Inferior {
                widget.set_opacity(0.6);
            }
            Inhibit(false)
        });

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
            widget: Self::create_row(&tag_box, assigned),
            eventbox: remove_event,
        };
        title_label.set_label(&tag.label);

        tag_row
    }

    fn create_row(widget: &Box, assigned: bool) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_activatable(!assigned);
        row.set_can_focus(false);
        row.add(widget);
        row.get_style_context().remove_class("activatable");

        row
    }
}
