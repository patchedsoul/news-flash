use crate::gtk_handle;
use crate::sidebar::feed_list::models::FeedListCategoryModel;
use crate::util::BuilderHelper;
use gdk::{EventMask, EventType};
use gtk::{
    self, BinExt, Box, Cast, ContainerExt, EventBox, Image, Label, LabelExt, ListBoxRow, ListBoxRowExt, Revealer,
    RevealerExt, StyleContextExt, WidgetExt, WidgetExtManual,
};
use news_flash::models::CategoryID;
use std::cell::RefCell;
use std::rc::Rc;
use std::str;

#[derive(Clone, Debug)]
pub struct CategoryRow {
    pub id: CategoryID,
    widget: ListBoxRow,
    revealer: Revealer,
    arrow_event: EventBox,
    item_count: Label,
    item_count_event: EventBox,
    title: Label,
    expanded: bool,
}

impl CategoryRow {
    pub fn new(model: &FeedListCategoryModel, visible: bool) -> Rc<RefCell<CategoryRow>> {
        let builder = BuilderHelper::new("category");
        let revealer = builder.get::<Revealer>("category_row");
        let level_margin = builder.get::<Box>("level_margin");
        level_margin.set_margin_start(model.level * 24);

        let title_label = builder.get::<Label>("category_title");
        let item_count_label = builder.get::<Label>("item_count");
        let item_count_event = builder.get::<EventBox>("item_count_event");
        let arrow_image = builder.get::<Image>("arrow_image");

        let arrow_event = builder.get::<EventBox>("arrow_event");
        let category = CategoryRow {
            id: model.id.clone(),
            widget: Self::create_row(&revealer),
            revealer,
            arrow_event: arrow_event.clone(),
            item_count: item_count_label,
            item_count_event,
            title: title_label,
            expanded: model.expanded,
        };
        category.update_title(&model.label);
        category.update_item_count(model.item_count);
        Self::rotate_arrow(&arrow_image.upcast::<gtk::Widget>(), model.expanded);
        if !visible {
            category.collapse();
        }
        let handle = gtk_handle!(category);
        let handle1 = handle.clone();

        arrow_event.set_events(EventMask::BUTTON_PRESS_MASK);
        arrow_event.set_events(EventMask::ENTER_NOTIFY_MASK);
        arrow_event.set_events(EventMask::LEAVE_NOTIFY_MASK);
        arrow_event.connect_enter_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(1.0);
            gtk::Inhibit(false)
        });
        arrow_event.connect_leave_notify_event(|widget, _| {
            widget.get_child().unwrap().set_opacity(0.8);
            gtk::Inhibit(false)
        });

        arrow_event.connect_button_press_event(move |widget, event| {
            if event.get_button() != 1 {
                return gtk::Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }
            let arrow_image = widget.get_child().unwrap();
            let expanded = handle1.borrow().expanded;
            Self::rotate_arrow(&arrow_image, !expanded);
            handle1.borrow_mut().expanded = !expanded;
            gtk::Inhibit(false)
        });

        handle
    }

    fn rotate_arrow(arrow_image: &gtk::Widget, expanded: bool) {
        let context = arrow_image.get_style_context();

        if expanded {
            context.remove_class("forward-arrow-collapsed");
            context.add_class("forward-arrow-expanded");
        } else {
            context.remove_class("forward-arrow-expanded");
            context.add_class("forward-arrow-collapsed");
        }
    }

    fn create_row(widget: &Revealer) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        let context = row.get_style_context();
        context.remove_class("activatable");

        row.add(widget);
        row
    }

    pub fn widget(&self) -> ListBoxRow {
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

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }

    pub fn expander_event(&self) -> EventBox {
        self.arrow_event.clone()
    }

    pub fn collapse(&self) {
        self.revealer.set_reveal_child(false);
        self.revealer.get_style_context().add_class("hidden");
        self.widget.set_selectable(false);
    }

    pub fn expand(&self) {
        self.revealer.set_reveal_child(true);
        self.revealer.get_style_context().remove_class("hidden");
        self.widget.set_selectable(true);
    }
}
