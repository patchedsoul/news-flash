use crate::util::BuilderHelper;
use gtk::{ContainerExt, Label, LabelExt, ListBoxRow, ListBoxRowExt, StyleContextExt, WidgetExt};

pub struct RelatedTopicRow {
    pub widget: ListBoxRow,
}

impl RelatedTopicRow {
    pub fn new(related_topic: &str, is_last: bool) -> Self {
        let builder = BuilderHelper::new("discover_dialog");
        let label = builder.get::<Label>("related_topic");

        label.set_label(&format!("# {}",related_topic));

        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(&label);
        row.show_all();
        let context = row.get_style_context();
        context.remove_class("activatable");
        if !is_last {
            context.add_class("search-item-separator");
        }

        RelatedTopicRow { widget: row }
    }
}
