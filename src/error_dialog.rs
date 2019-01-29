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
            let mut string = format!("{}", cause);
            
            if let Some(error) = NewsFlash::parse_error(cause) {
                string = format!("{} ({})", string, error);
            }

            let index_label = gtk::Label::new(None);
            index_label.set_text(&format!("{}", i,));
            index_label.set_size_request(30, 0);

            let message_label = gtk::Label::new(None);
            message_label.set_text(&string);
            message_label.set_ellipsize(pango::EllipsizeMode::End);
            message_label.set_xalign(0.0);

            let v_separator = gtk::Separator::new(gtk::Orientation::Vertical);

            let h_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            h_box.set_size_request(0, 30);
            h_box.set_margin_start(5);
            h_box.set_margin_end(5);
            h_box.pack_start(&index_label, false, true, 0);
            h_box.pack_start(&v_separator, false, true, 0);
            h_box.pack_start(&message_label, false, true, 0);
            
            
            let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
            list_box.pack_start(&h_box, false, true, 0);
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