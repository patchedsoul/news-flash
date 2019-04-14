use crate::Resources;
use failure::format_err;
use failure::Error;
use gtk::{LabelExt, WidgetExt};
use std::str;

#[derive(Clone, Debug)]
pub struct ProgressOverlay {
    parent: gtk::Box,
    label: gtk::Label,
}

impl ProgressOverlay {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_view_progress.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let label: gtk::Label = builder.get_object("label").ok_or(format_err!("some err"))?;
        let parent: gtk::Box = builder.get_object("box").ok_or(format_err!("some err"))?;

        Ok(ProgressOverlay { parent: parent, label: label })
    }

    pub fn set_percentage(&self, percentage: f64) {
        let mut percentage = percentage;
        if percentage < 0.0 {
            percentage = 0.0;
        }
        if percentage > 1.0 {
            percentage = 1.0;
        }
        self.label.set_label(&format!("{}%", (percentage * 100.0) as i32));
    }

    pub fn reveal(&self, show: bool) {
        match show {
            true => {
                self.parent.show();
            }
            false => {
                self.parent.hide();
            }
        }
    }

    pub fn widget(&self) -> gtk::Box {
        self.parent.clone()
    }
}
