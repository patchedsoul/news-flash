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
    Feed as FeedModel,
};
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use Resources;

#[derive(Clone, Debug)]
pub struct Feed {
    pub id: FeedID,
    widget: gtk::ListBoxRow,
    revealer: gtk::Revealer,
}

impl Feed {
    pub fn new(model: &FeedModel) -> Rc<RefCell<Feed>> {
        let ui_data = Resources::get("ui/feed.ui").unwrap();
        let ui_string = str::from_utf8(&ui_data).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let feed : gtk::Revealer = builder.get_object("feed_row").unwrap();
        
        let label_widget : gtk::Label = builder.get_object("feed_title").unwrap();
        label_widget.set_label(&model.label);

        let feed = Feed {
            id: model.feed_id.clone(),
            widget: Self::create_row(&feed),
            revealer: feed,
        };
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
