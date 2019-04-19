use crate::Resources;
use gtk::{LabelExt, WidgetExt};
use crate::util::{GTK_RESOURCE_FILE_ERROR, GTK_BUILDER_ERROR};
use std::str;

#[derive(Clone, Debug)]
pub struct ProgressOverlay {
    parent: gtk::Box,
    label: gtk::Label,
}

impl ProgressOverlay {
    pub fn new() -> Self {
        let ui_data = Resources::get("ui/article_view_progress.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);

        let builder = gtk::Builder::new_from_string(ui_string);
        let label: gtk::Label = builder.get_object("label").expect(GTK_BUILDER_ERROR);
        let parent: gtk::Box = builder.get_object("box").expect(GTK_BUILDER_ERROR);

        ProgressOverlay { parent, label }
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
        if show {
            self.parent.show();
        } else {
            self.parent.hide();
        }
    }

    pub fn widget(&self) -> gtk::Box {
        self.parent.clone()
    }
}
