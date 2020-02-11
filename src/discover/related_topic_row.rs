use crate::util::BuilderHelper;
use glib::clone;
use gtk::{Button, ButtonExt, ContainerExt, EntryExt, FlowBoxChild, SearchEntry, WidgetExt};

pub struct RelatedTopicRow {
    pub widget: FlowBoxChild,
}

impl RelatedTopicRow {
    pub fn new(related_topic: &str, search_entry: &SearchEntry) -> Self {
        let builder = BuilderHelper::new("discover_dialog");
        let button = builder.get::<Button>("related_topic");

        button.set_label(&format!("# {}", related_topic));

        let topic_name_string = related_topic.to_owned();
        button.connect_clicked(clone!(
            @weak search_entry => @default-panic, move |_button|
        {
            search_entry.set_text(&format!("#{}", topic_name_string));
        }));

        let row = FlowBoxChild::new();
        row.set_can_focus(false);
        row.add(&button);
        row.show_all();

        RelatedTopicRow { widget: row }
    }
}
