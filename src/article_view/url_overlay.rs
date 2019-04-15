use crate::Resources;
use failure::format_err;
use failure::Error;
use gtk::{LabelExt, WidgetExt};
use std::str;

#[derive(Clone, Debug)]
pub struct UrlOverlay {
    label: gtk::Label,
}

impl UrlOverlay {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_view_url.ui").ok_or_else(|| format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let label: gtk::Label = builder.get_object("label").ok_or_else(|| format_err!("some err"))?;

        Ok(UrlOverlay { label })
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
