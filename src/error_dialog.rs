use crate::Resources;
use failure::Fail;
use gtk::{BoxExt, GtkWindowExt, LabelExt, WidgetExt};
use news_flash::{NewsFlash, NewsFlashError};
use std::str;
use crate::util::{GTK_RESOURCE_FILE_ERROR, GTK_BUILDER_ERROR};


#[derive(Clone, Debug)]
pub struct ErrorDialog {
    error_dialog: gtk::Window,
    list_box: gtk::Box,
}

impl ErrorDialog {
    pub fn new(error: &NewsFlashError, parent: &gtk::ApplicationWindow) -> Self {
        let ui_data = Resources::get("ui/error_detail_dialog.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);
        let builder = gtk::Builder::new_from_string(ui_string);
        let list_box: gtk::Box = builder.get_object("list_box").expect(GTK_BUILDER_ERROR);
        let error_dialog: gtk::Window = builder.get_object("error_dialog").expect(GTK_BUILDER_ERROR);

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

        error_dialog.set_transient_for(parent);
        error_dialog.show_all();

        ErrorDialog {
            error_dialog,
            list_box,
        }
    }
}
