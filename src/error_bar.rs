use crate::app::Action;
use crate::error_dialog::ErrorDialog;
use crate::util::{BuilderHelper, GtkUtil, Util};
use glib::{translate::ToGlib, Sender};
use gtk::{Button, ButtonExt, InfoBar, InfoBarExt, Label, LabelExt, ResponseType, WidgetExt};
use log::error;
use news_flash::{NewsFlash, NewsFlashError};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ErrorBar {
    widget: InfoBar,
    label: Label,
    button: Button,
    login_button: Button,
    click_signal: Arc<RwLock<Option<u64>>>,
    relogin_signal: Arc<RwLock<Option<u64>>>,
    sender: Sender<Action>,
}

impl ErrorBar {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let error_bar = ErrorBar {
            widget: builder.get::<InfoBar>("error_bar"),
            label: builder.get::<Label>("error_label"),
            button: builder.get::<Button>("info_button"),
            login_button: builder.get::<Button>("relogin_button"),
            click_signal: Arc::new(RwLock::new(None)),
            relogin_signal: Arc::new(RwLock::new(None)),
            sender,
        };

        error_bar.init();

        error_bar
    }

    fn init(&self) {
        self.widget.set_visible(true);
        self.widget.set_revealed(false);

        let click_signal = self.click_signal.clone();
        let relogin_signal = self.relogin_signal.clone();
        let button = self.button.clone();
        let login_button = self.login_button.clone();
        self.widget.connect_response(move |info_bar, response_type| {
            if response_type == ResponseType::Close {
                info_bar.set_revealed(false);
                GtkUtil::disconnect_signal(*click_signal.read(), &button);
                GtkUtil::disconnect_signal(*relogin_signal.read(), &login_button);
                click_signal.write().take();
                relogin_signal.write().take();
            }
        });
    }

    pub fn hide(&self) {
        self.widget.set_revealed(false);
    }

    pub fn simple_message(&self, message: &str) {
        self.label.set_text(message);
        self.widget.set_revealed(true);
        self.button.set_visible(false);
        self.login_button.set_visible(false);
    }

    pub fn news_flash_error(&self, message: &str, error: NewsFlashError) {
        self.label.set_text(message);
        self.widget.set_revealed(true);
        self.button.set_visible(true);
        
        let show_login_button = NewsFlash::error_login_related(&error);
        self.login_button.set_visible(show_login_button);
        GtkUtil::disconnect_signal(*self.relogin_signal.read(), &self.login_button);

        if show_login_button {
            let sender = self.sender.clone();
            *self.relogin_signal.write() = Some(
                self.login_button
                    .connect_clicked(move |_button| {
                        log::info!("retry login");
                        Util::send(&sender, Action::RetryLogin);
                    })
                    .to_glib(),
            );
        }

        GtkUtil::disconnect_signal(*self.click_signal.read(), &self.button);
        *self.click_signal.write() = Some(
            self.button
                .connect_clicked(move |button| {
                    if let Ok(parent) = GtkUtil::get_main_window(button) {
                        let _dialog = ErrorDialog::new(&error, &parent);
                    } else {
                        error!("Failed to spawn ErrorDialog. Parent window not found.");
                    }
                })
                .to_glib(),
        );
    }
}
