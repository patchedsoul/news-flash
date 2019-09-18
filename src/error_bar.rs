use crate::error_dialog::ErrorDialog;
use crate::util::{BuilderHelper, GtkUtil};
use gtk::{Button, ButtonExt, InfoBar, InfoBarExt, Label, LabelExt, ResponseType, WidgetExt};
use news_flash::NewsFlashError;

#[derive(Clone, Debug)]
pub struct ErrorBar {
    widget: InfoBar,
    label: Label,
    button: Button,
}

impl ErrorBar {
    pub fn new(builder: &BuilderHelper) -> Self {
        let error_bar = ErrorBar {
            widget: builder.get::<InfoBar>("error_bar"),
            label: builder.get::<Label>("error_label"),
            button: builder.get::<Button>("info_button"),
        };

        error_bar.init();

        error_bar
    }

    fn init(&self) {
        self.widget.set_visible(true);
        self.widget.set_revealed(false);

        self.widget.connect_response(|info_bar, response_type| {
            if response_type == ResponseType::Close {
                info_bar.set_revealed(false);
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

        self.button.connect_clicked(move |button| {
            let parent = GtkUtil::get_main_window(button).unwrap();
            let _dialog = ErrorDialog::new(&error, &parent);
        });
    }
}
