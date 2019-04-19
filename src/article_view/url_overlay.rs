use gtk::{Label, LabelExt, WidgetExt};
use crate::util::BuilderHelper;

#[derive(Clone, Debug)]
pub struct UrlOverlay {
    label: gtk::Label,
}

impl UrlOverlay {
    pub fn new() -> Self {
        let builder = BuilderHelper::new("article_view_url");
        let label = builder.get::<Label>("label");

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
