use gtk::{
    self,
    LabelExt,
    WidgetExt,
    RevealerExt,
    StyleContextExt,
    BinExt,
};
use gdk::{
    EventMask,
    EventType,
};
use news_flash::models::{
    CategoryID,
};
use std::str;
use Resources;

#[derive(Clone, Debug)]
pub struct Category {
    pub id: CategoryID,
    pub(crate) widget: gtk::Box,
}

impl Category {
    pub fn new(id: CategoryID, label: &str) -> Self {
        let ui_data = Resources::get("ui/category.ui").unwrap();
        let ui_string = str::from_utf8(&ui_data).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let category : gtk::Box = builder.get_object("category_row").unwrap();
        let category_revealer : gtk::Revealer = builder.get_object("category_revealer").unwrap();
        
        let label_widget : gtk::Label = builder.get_object("category_title").unwrap();
        label_widget.set_label(label);

        let arrow_event : gtk::EventBox = builder.get_object("arrow_event").unwrap();
        arrow_event.set_events(EventMask::BUTTON_PRESS_MASK.bits() as i32);
        arrow_event.set_events(EventMask::ENTER_NOTIFY_MASK.bits() as i32);
        arrow_event.set_events(EventMask::LEAVE_NOTIFY_MASK.bits() as i32);
        arrow_event.connect_enter_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(1.0);
            gtk::Inhibit(false)
        });
        arrow_event.connect_leave_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(0.8);
            gtk::Inhibit(false)
        });

        arrow_event.connect_button_press_event(move |widget, event| {
            if event.get_event_type() == EventType::ButtonPress {
                let arrow_image = widget.get_child().unwrap();
                let context = arrow_image.get_style_context().unwrap();
                let expanded = context.has_class("expanded");

                if expanded {
                    context.remove_class("expanded");
                    context.add_class("collapsed");
                    category_revealer.set_reveal_child(false);
                }
                else {
                    context.add_class("expanded");
                    context.remove_class("collapsed");
                    category_revealer.set_reveal_child(true);
                }
            }
            gtk::Inhibit(false)
        });

        Category {
            id: id,
            widget: category,
        }
    }
}
