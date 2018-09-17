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
    Category as CategoryModel,
    CategoryID,
};
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use Resources;

#[derive(Clone, Debug)]
pub struct CategoryRow {
    pub id: CategoryID,
    pub parent: CategoryID,
    widget: gtk::ListBoxRow,
    revealer: gtk::Revealer,
    arrow_event: gtk::EventBox,
    expanded: bool,
    sort_index: Option<i32>,
}

impl CategoryRow {
    pub fn new(model: &CategoryModel, level: i32) -> Rc<RefCell<CategoryRow>> {
        let ui_data = Resources::get("ui/category.ui").unwrap();
        let ui_string = str::from_utf8(&ui_data).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let category : gtk::Revealer = builder.get_object("category_row").unwrap();
        category.set_margin_start(level*24);
        
        let label_widget : gtk::Label = builder.get_object("category_title").unwrap();
        label_widget.set_label(&model.label);

        let arrow_image : gtk::Image = builder.get_object("arrow_image").unwrap();
        arrow_image.get_style_context().unwrap().add_class("expanded");

        let arrow_event : gtk::EventBox = builder.get_object("arrow_event").unwrap();
        let category = CategoryRow {
            id: model.category_id.clone(),
            parent: model.parent.clone(),
            widget: Self::create_row(&category),
            revealer: category,
            arrow_event: arrow_event.clone(),
            expanded: true,
            sort_index: model.sort_index,
        };
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

    pub fn expander_event(&self) -> gtk::EventBox {
        self.arrow_event.clone()
    }

    pub fn is_expaneded(&self) -> bool {
        self.expanded
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

    pub fn sort_index(&self) -> Option<i32> {
        self.sort_index
    }
}
