use gtk::{
    self,
    LabelExt,
    ContainerExt,
    WidgetExt,
    StyleContextExt,
    ListBoxRowExt,
    RevealerExt,
};
use news_flash::models::{
    FeedID,
};
use sidebar::feed_list::models::{
    FeedListFeedModel,
};
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use Resources;

#[derive(Clone, Debug)]
pub struct FeedRow {
    pub id: FeedID,
    widget: gtk::ListBoxRow,
    item_count: gtk::Label,
    item_count_event: gtk::EventBox,
    title: gtk::Label,
    revealer: gtk::Revealer,
}

impl FeedRow {
    pub fn new(model: &FeedListFeedModel) -> Rc<RefCell<FeedRow>> {
        let ui_data = Resources::get("ui/feed.ui").unwrap();
        let ui_string = str::from_utf8(&ui_data).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let feed : gtk::Revealer = builder.get_object("feed_row").unwrap();
        feed.set_margin_start(model.level*24);
        
        let title_label : gtk::Label = builder.get_object("feed_title").unwrap();
        let item_count_label : gtk::Label = builder.get_object("item_count").unwrap();
        let item_count_event : gtk::EventBox = builder.get_object("item_count_event").unwrap();

        let feed = FeedRow {
            id: model.id.clone(),
            widget: Self::create_row(&feed),
            item_count: item_count_label,
            title: title_label,
            revealer: feed,
            item_count_event: item_count_event,
        };
        feed.update_item_count(model.item_count);
        feed.update_title(&model.label);
        Rc::new(RefCell::new(feed))
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
