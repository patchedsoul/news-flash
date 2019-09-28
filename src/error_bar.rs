use crate::error_dialog::ErrorDialog;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
use crate::gtk_handle;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::{Button, ButtonExt, InfoBar, InfoBarExt, Label, LabelExt, ResponseType, WidgetExt};
use glib::translate::ToGlib;
use log::error;
use news_flash::NewsFlashError;

#[derive(Clone, Debug)]
pub struct ErrorBar {
    widget: InfoBar,
    label: Label,
    button: Button,
    click_signal: GtkHandle<Option<u64>>,
}

impl ErrorBar {
    pub fn new(builder: &BuilderHelper) -> Self {
        let error_bar = ErrorBar {
            widget: builder.get::<InfoBar>("error_bar"),
            label: builder.get::<Label>("error_label"),
            button: builder.get::<Button>("info_button"),
            click_signal: gtk_handle!(None),
        };

        error_bar.init();

        error_bar
    }

    fn init(&self) {
        self.widget.set_visible(true);
        self.widget.set_revealed(false);

        let click_signal = self.click_signal.clone();
        let button = self.button.clone();
        self.widget.connect_response(move |info_bar, response_type| {
            if response_type == ResponseType::Close {
                info_bar.set_revealed(false);
                GtkUtil::disconnect_signal_handle(&click_signal, &button);
            }
        });
    }

    pub fn simple_message(&self, message: &str) {
        self.label.set_text(message);
        self.widget.set_revealed(true);
        self.button.set_visible(false);
    }

    pub fn news_flash_error(&self, message: &str, error: NewsFlashError) {
        self.label.set_text(message);
        self.widget.set_revealed(true);
        self.button.set_visible(true);

        GtkUtil::disconnect_signal_handle(&self.click_signal, &self.button);

        self.click_signal.replace(Some(self.button.connect_clicked(move |button| {
            if let Ok(parent) = GtkUtil::get_main_window(button) {
                let _dialog = ErrorDialog::new(&error, &parent);
            } else {
                error!("Failed to spawn ErrorDialog. Parent window not found.");
            }
        }).to_glib()));
    }
}
