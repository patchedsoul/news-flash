use news_flash::NewsFlashError;
use crate::Resources;
use std::str;
use failure::{
    Error,
    format_err,
};
use gtk::{
    TextViewExt,
    TextBufferExt,
    GtkWindowExt,
    WidgetExt,
};

#[derive(Clone, Debug)]
pub struct ErrorDialog {
    error_dialog: gtk::Window,
    text_view: gtk::TextView,
}

impl ErrorDialog {
    pub fn new(error: &NewsFlashError, parent: gtk::ApplicationWindow) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/error_detail_dialog.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let text_view : gtk::TextView = builder.get_object("text_view").ok_or(format_err!("some err"))?;
        let error_dialog : gtk::Window = builder.get_object("error_dialog").ok_or(format_err!("some err"))?;
        error_dialog.set_transient_for(&parent);
        error_dialog.show_all();

        let buffer = text_view.get_buffer().ok_or(format_err!("some err"))?;
        buffer.set_text(&format!("{:?}", error));

        Ok(ErrorDialog{
            error_dialog: error_dialog,
            text_view: text_view,
        })
    }
}