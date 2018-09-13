use gtk::{
    self,
    LabelExt,
};
use news_flash::models::{
    CategoryID,
    FeedID,
    Feed as FeedModel,
};
use std::str;
use Resources;

#[derive(Clone, Debug)]
pub struct Feed {
    pub id: FeedID,
    pub parent: CategoryID,
    pub(crate) widget: gtk::Box,
}

impl Feed {
    pub fn new(model: &FeedModel, parent: &CategoryID) -> Self {
        let ui_data = Resources::get("ui/feed.ui").unwrap();
        let ui_string = str::from_utf8(&ui_data).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let feed : gtk::Box = builder.get_object("feed_row").unwrap();
        
        let label_widget : gtk::Label = builder.get_object("feed_title").unwrap();
        label_widget.set_label(&model.label);

        Feed {
            id: model.feed_id.clone(),
            parent: parent.clone(),
            widget: feed,
        }
    }
}
