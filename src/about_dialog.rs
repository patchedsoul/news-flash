use gtk::{AboutDialog, AboutDialogExt, License, Window, GtkWindowExt};
use glib::{object::IsA};

pub const APP_NAME: &str = "NewsFlash";
pub const COPYRIGHT: &str = "Copyright © 2017-2019 Jan Lukas Gernert";
pub const VERSION: &str = "0.1";
pub const DESCRIPTION: &str = "Desktop Client for various RSS Services";
pub const AUTHORS: &'static [&str] = &["Jan Lukas Gernert", "Brendan Long"];
pub const ICON_NAME: &str = "com.gitlab.newsflash";

#[derive(Clone, Debug)]
pub struct NewsFlashAbout {
    widget: AboutDialog,
}

impl NewsFlashAbout {
    pub fn new<W: IsA<Window> + GtkWindowExt>(window: Option<&W>) -> Self {
        let widget = AboutDialog::new();
        widget.set_transient_for(window);
        widget.set_modal(true);
        widget.set_authors(AUTHORS);
        widget.set_comments(DESCRIPTION);
        widget.set_copyright(COPYRIGHT);
        widget.set_logo_icon_name(ICON_NAME);
        widget.set_program_name(APP_NAME);
        widget.set_version(VERSION);
        widget.set_license_type(License::Gpl30);
        widget.set_wrap_license(true);

        NewsFlashAbout {
            widget,
        }
    }

    pub fn widget(&self) -> AboutDialog {
        self.widget.clone()
    }
}