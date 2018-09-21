use gtk::{
    self,
    LabelExt,
    WidgetExt,
    StyleContextExt,
    BinExt,
    ListBoxRowExt,
    ContainerExt,
    RevealerExt,
};
use gdk::{
    EventMask,
    EventType,
};
use news_flash::models::{
    CategoryID,
};
use sidebar::feed_list::models::{
    FeedListCategoryModel,
};
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use Resources;

#[derive(Clone, Debug)]
pub struct CategoryRow {
    pub id: CategoryID,
    widget: gtk::ListBoxRow,
    revealer: gtk::Revealer,
    arrow_event: gtk::EventBox,
    item_count: gtk::Label,
    item_count_event: gtk::EventBox,
    title: gtk::Label,
    expanded: bool,
}

impl CategoryRow {
    pub fn new(model: &FeedListCategoryModel, visible: bool) -> Rc<RefCell<CategoryRow>> {
        let ui_data = Resources::get("ui/category.ui").unwrap();
        let ui_string = str::from_utf8(&ui_data).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let category : gtk::Revealer = builder.get_object("category_row").unwrap();
        category.set_margin_start(model.level*24);
        
        let title_label : gtk::Label = builder.get_object("category_title").unwrap();
        let item_count_label : gtk::Label = builder.get_object("item_count").unwrap();
        let item_count_event : gtk::EventBox = builder.get_object("item_count_event").unwrap();

        let arrow_image : gtk::Image = builder.get_object("arrow_image").unwrap();
        arrow_image.get_style_context().unwrap().add_class("expanded");

        let arrow_event : gtk::EventBox = builder.get_object("arrow_event").unwrap();
        let category = CategoryRow {
            id: model.id.clone(),
            widget: Self::create_row(&category),
            revealer: category,
            arrow_event: arrow_event.clone(),
            item_count: item_count_label,
            item_count_event: item_count_event,
            title: title_label,
            expanded: model.expanded,
        };
        category.update_title(&model.label);
        category.update_item_count(model.item_count);
        if !visible {
            category.collapse();
        }
        let handle = Rc::new(RefCell::new(category));
        let handle1 = handle.clone();

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
                let mut category = handle1.borrow_mut();

                if category.expanded {
                    context.remove_class("expanded");
                    context.add_class("collapsed");
                    category.expanded = false;
                }
                else {
                    context.add_class("expanded");
                    context.remove_class("collapsed");
                    category.expanded = true;
                }
            }
            gtk::Inhibit(false)
        });

        handle
    }

    fn create_row(widget: &gtk::Revealer) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(false);
        row.set_can_focus(false);
        let context = row.get_style_context().unwrap();
        context.remove_class("activatable");
        
        row.add(widget);
        row
    }

    pub fn row(&self) -> gtk::ListBoxRow {
        self.widget.clone()
    }

    pub fn update_item_count(&self, count: i32) {
        if count > 0 {
            self.item_count.set_label(&count.to_string());
            self.item_count_event.set_visible(true);
        }
        else {
            self.item_count_event.set_visible(false);
        }
    }

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }

    pub fn expander_event(&self) -> gtk::EventBox {
        self.arrow_event.clone()
    }

    pub fn collapse(&self) {
        self.revealer.set_reveal_child(false);
        self.revealer.get_style_context().unwrap().add_class("hidden");
        self.widget.set_selectable(false);
    }

    pub fn expand(&self) {
        self.revealer.set_reveal_child(true);
        self.revealer.get_style_context().unwrap().remove_class("hidden");
        self.widget.set_selectable(true);
    }
}
