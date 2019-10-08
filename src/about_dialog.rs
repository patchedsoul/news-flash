use crate::config::{APP_ID, VERSION};
use glib::object::IsA;
use gtk::{AboutDialog, AboutDialogExt, GtkWindowExt, License, Window};

pub const APP_NAME: &str = "NewsFlash";
pub const COPYRIGHT: &str = "Copyright © 2017-2019 Jan Lukas Gernert";
pub const DESCRIPTION: &str = "Desktop Client for various RSS Services";
pub const AUTHORS: &[&str] = &["Jan Lukas Gernert", "Brendan Long"];

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
        widget.set_comments(Some(DESCRIPTION));
        widget.set_copyright(Some(COPYRIGHT));
        widget.set_logo_icon_name(Some(APP_ID));
        widget.set_program_name(APP_NAME);
        widget.set_version(Some(VERSION));
        widget.set_license_type(License::Gpl30);
        widget.set_wrap_license(true);

        NewsFlashAbout { widget }
    }

    pub fn widget(&self) -> AboutDialog {
        self.widget.clone()
    }
}
