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
    detail_button: Button,
    login_button: Button,
    offline_button: Button,
    detail_signal: Arc<RwLock<Option<u64>>>,
    relogin_signal: Arc<RwLock<Option<u64>>>,
    offline_signal: Arc<RwLock<Option<u64>>>,
    sender: Sender<Action>,
}

impl ErrorBar {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let error_bar = ErrorBar {
            widget: builder.get::<InfoBar>("error_bar"),
            label: builder.get::<Label>("error_label"),
            detail_button: builder.get::<Button>("info_button"),
            login_button: builder.get::<Button>("relogin_button"),
            offline_button: builder.get::<Button>("offline_button"),
            detail_signal: Arc::new(RwLock::new(None)),
            relogin_signal: Arc::new(RwLock::new(None)),
            offline_signal: Arc::new(RwLock::new(None)),
            sender,
        };

        error_bar.init();

        error_bar
    }

    fn init(&self) {
        self.widget.set_visible(true);
        self.widget.set_revealed(false);

        let detail_signal = self.detail_signal.clone();
        let relogin_signal = self.relogin_signal.clone();
        let detail_button = self.detail_button.clone();
        let login_button = self.login_button.clone();
        let offline_button = self.offline_button.clone();
        let offline_signal = self.offline_signal.clone();
        self.widget.connect_response(move |info_bar, response_type| {
            if response_type == ResponseType::Close {
                Self::close(
                    &info_bar,
                    &detail_button,
                    &login_button,
                    &offline_button,
                    &detail_signal,
                    &relogin_signal,
                    &offline_signal,
                );
            }
        });
    }

    fn close(
        info_bar: &InfoBar,
        detail_button: &Button,
        login_button: &Button,
        offline_button: &Button,
        detail_signal: &Arc<RwLock<Option<u64>>>,
        relogin_signal: &Arc<RwLock<Option<u64>>>,
        offline_signal: &Arc<RwLock<Option<u64>>>,
    ) {
        info_bar.set_revealed(false);
        GtkUtil::disconnect_signal(*detail_signal.read(), detail_button);
        GtkUtil::disconnect_signal(*relogin_signal.read(), login_button);
        GtkUtil::disconnect_signal(*offline_signal.read(), offline_button);
        detail_signal.write().take();
        relogin_signal.write().take();
        offline_signal.write().take();
    }

    pub fn hide(&self) {
        self.widget.set_revealed(false);
    }

    pub fn simple_message(&self, message: &str) {
        self.label.set_text(message);
        self.widget.set_revealed(true);
        self.detail_button.set_visible(false);
        self.login_button.set_visible(false);
        self.offline_button.set_visible(false);
    }

    pub fn news_flash_error(&self, message: &str, error: NewsFlashError) {
        self.label.set_text(message);
        self.widget.set_revealed(true);
        self.detail_button.set_visible(true);

        let show_login_button = NewsFlash::error_login_related(&error);
        self.login_button.set_visible(show_login_button);
        self.offline_button.set_visible(show_login_button);
        GtkUtil::disconnect_signal(*self.relogin_signal.read(), &self.login_button);
        GtkUtil::disconnect_signal(*self.offline_signal.read(), &self.offline_button);

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

            let sender = self.sender.clone();
            let info_bar = self.widget.clone();
            let detail_signal = self.detail_signal.clone();
            let relogin_signal = self.relogin_signal.clone();
            let detail_button = self.detail_button.clone();
            let login_button = self.login_button.clone();
            let offline_button = self.offline_button.clone();
            let offline_signal = self.offline_signal.clone();
            *self.offline_signal.write() = Some(
                self.offline_button
                    .connect_clicked(move |_button| {
                        log::info!("switch into offline mode");
                        Self::close(
                            &info_bar,
                            &detail_button,
                            &login_button,
                            &offline_button,
                            &detail_signal,
                            &relogin_signal,
                            &offline_signal,
                        );
                        Util::send(&sender, Action::SetOfflineMode(true));
                    })
                    .to_glib(),
            );
        }

        GtkUtil::disconnect_signal(*self.detail_signal.read(), &self.detail_button);
        *self.detail_signal.write() = Some(
            self.detail_button
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
