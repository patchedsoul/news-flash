use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use gtk::{
    LabelExt,
    WidgetExt,
    RevealerExt,
    Continue,
};

#[derive(Clone, Debug)]
pub struct UrlOverlay {
    revealer: gtk::Revealer,
    label: gtk::Label,
}

impl UrlOverlay {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_view_url.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let revealer : gtk::Revealer = builder.get_object("revealer").ok_or(format_err!("some err"))?;
        let label : gtk::Label = builder.get_object("label").ok_or(format_err!("some err"))?;

        Ok(UrlOverlay {
            revealer: revealer,
            label: label,
        })
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
        self.revealer.set_halign(align);
    }

    pub fn reveal(&self, show: bool) {
        match show {
            true => {
                self.revealer.set_visible(true);
                self.revealer.set_reveal_child(true);
                self.label.show();
            },
            false => {
                self.revealer.set_reveal_child(false);
                let revealer_clone = self.revealer.clone();
                gtk::timeout_add(150, move || {
                    revealer_clone.set_visible(false);
                    Continue(false)
                });
            },
        }
    }

    pub fn widget(&self) -> gtk::Revealer {
        self.revealer.clone()
    }
}