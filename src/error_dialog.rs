use news_flash::{
    NewsFlash,
    NewsFlashError,
};
use crate::Resources;
use std::str;
use failure::{
    Fail,
    Error,
    format_err,
};
use gtk::{
    BoxExt,
    LabelExt,
    GtkWindowExt,
    WidgetExt,
};

#[derive(Clone, Debug)]
pub struct ErrorDialog {
    error_dialog: gtk::Window,
    list_box: gtk::Box,
}

impl ErrorDialog {
    pub fn new(error: &NewsFlashError, parent: gtk::ApplicationWindow) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/error_detail_dialog.ui").ok_or(format_err!("some err1"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let list_box : gtk::Box = builder.get_object("list_box").ok_or(format_err!("some err2"))?;
        let error_dialog : gtk::Window = builder.get_object("error_dialog").ok_or(format_err!("some err3"))?;

        for (i, cause) in Fail::iter_chain(error).enumerate() {
            let mut string = format!("{}: {}", i, cause);
            
            if let Some(error) = NewsFlash::parse_error(cause) {
                string = format!("{} ({})", string, error);
            }

            let label = gtk::Label::new(None);
            label.set_text(&string);
            label.set_size_request(0, 30);
            label.set_ellipsize(pango::EllipsizeMode::End);
            label.set_xalign(0.0);
            label.set_margin_start(5);
            label.set_margin_end(5);
            let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
            list_box.pack_start(&label, false, true, 0);
            list_box.pack_start(&separator, false, true, 0);
        }

        error_dialog.set_transient_for(&parent);
        error_dialog.show_all();

        Ok(ErrorDialog{
            error_dialog: error_dialog,
            list_box: list_box,
        })
    }
}