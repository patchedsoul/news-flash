use crate::util::BuilderHelper;
use gtk::{Box, Label, LabelExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct ProgressOverlay {
    parent: gtk::Box,
    label: gtk::Label,
}

impl ProgressOverlay {
    pub fn new() -> Self {
        let builder = BuilderHelper::new("article_view_progress");
        let label = builder.get::<Label>("label");
        let parent = builder.get::<Box>("box");

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
