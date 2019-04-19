use crate::Resources;
use gtk::{LabelExt, WidgetExt};
use crate::util::{GTK_RESOURCE_FILE_ERROR, GTK_BUILDER_ERROR};
use std::str;

#[derive(Clone, Debug)]
pub struct UrlOverlay {
    label: gtk::Label,
}

impl UrlOverlay {
    pub fn new() -> Self {
        let ui_data = Resources::get("ui/article_view_url.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);

        let builder = gtk::Builder::new_from_string(ui_string);
        let label: gtk::Label = builder.get_object("label").expect(GTK_BUILDER_ERROR);

        UrlOverlay { label }
    }

    pub fn set_url(&self, uri: String, align: gtk::Align) {
        let max_length = 45;
        let mut uri = uri.clone();
        if uri.chars().count() > max_length {
            uri = uri.chars().take(max_length).collect::<String>();
            uri.push_str("...");
        }

        self.label.set_label(&uri);
        self.label.set_width_chars(uri.chars().count() as i32 - 5);
        self.label.set_halign(align);
    }

    pub fn reveal(&self, show: bool) {
        if show {
            self.label.show();
        } else {
            self.label.hide();
        }
    }

    pub fn widget(&self) -> gtk::Label {
        self.label.clone()
    }
}
