use crate::util::BuilderHelper;
use gtk::{Box, ProgressBar, ProgressBarExt, WidgetExt};

#[derive(Clone, Debug)]
pub struct ProgressOverlay {
    parent: Box,
    progress: ProgressBar,
}

impl ProgressOverlay {
    pub fn new() -> Self {
        let builder = BuilderHelper::new("article_view_progress");
        let progress = builder.get::<ProgressBar>("progress");
        let parent = builder.get::<Box>("box");

        ProgressOverlay { parent, progress }
    }

    pub fn set_percentage(&self, percentage: f64) {
        let mut percentage = percentage;
        if percentage < 0.0 {
            percentage = 0.0;
        }
        if percentage > 1.0 {
            percentage = 1.0;
        }
        self.progress.set_fraction(percentage);
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
