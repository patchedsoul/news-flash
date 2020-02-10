use crate::util::BuilderHelper;
use feedly_api::models::SearchResultItem;
use gtk::{Box, ContainerExt, Label, LabelExt, ListBoxRow, ListBoxRowExt, StyleContextExt, WidgetExt};

pub struct SearchItemRow {
    pub widget: ListBoxRow,
}

impl SearchItemRow {
    pub fn new(item: &SearchResultItem, is_last: bool) -> Self {
        let builder = BuilderHelper::new("discover_dialog");
        let search_item_row = builder.get::<Box>("search_item_row");
        let search_item_title = builder.get::<Label>("search_item_title");
        let search_item_description = builder.get::<Label>("search_item_description");

        search_item_title.set_label(&item.title);
        if let Some(description) = &item.description {
            search_item_description.set_label(description);
        }

        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(&search_item_row);
        row.show_all();
        let context = row.get_style_context();
        context.remove_class("activatable");
        if !is_last {
            context.add_class("search-item-separator");
        }

        SearchItemRow { widget: row }
    }
}
