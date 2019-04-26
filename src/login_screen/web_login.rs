use crate::error_dialog::ErrorDialog;
use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil, GTK_BUILDER_ERROR};
use failure::Error;
use gio::{ActionExt, ActionMapExt};
use glib::{
    signal::SignalHandlerId,
    translate::{FromGlib, ToGlib},
    Variant,
};
use gtk::{Box, BoxExt, Button, ButtonExt, InfoBar, InfoBarExt, Label, LabelExt, ObjectExt, ResponseType, WidgetExt};
use news_flash::models::{LoginData, LoginGUI, OAuthData, PluginInfo};
use news_flash::{NewsFlashError, NewsFlashErrorKind};
use std::cell::RefCell;
use std::rc::Rc;
use webkit2gtk::{LoadEvent, UserContentManager, WebContext, WebView, WebViewExt, WebViewExtManual};

#[derive(Clone, Debug)]
pub struct WebLogin {
    webview: WebView,
    page: gtk::Box,
    info_bar: gtk::InfoBar,
    info_bar_label: gtk::Label,
    error_details_button: gtk::Button,
    redirect_signal_id: GtkHandle<Option<u64>>,
    info_bar_close_signal: Option<u64>,
    info_bar_response_signal: Option<u64>,
    error_details_signal: Option<u64>,
}

impl WebLogin {
    pub fn new() -> Self {
        let builder = BuilderHelper::new("oauth_login");
        let page = builder.get::<Box>("oauth_box");
        let info_bar = builder.get::<InfoBar>("info_bar");
        let error_details_button = builder.get::<Button>("details_button");
        let info_bar_label = builder.get::<Label>("info_bar_label");

        let context = WebContext::get_default().expect(GTK_BUILDER_ERROR);
        let content_manager = UserContentManager::new();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &content_manager);

        page.pack_start(&webview, true, true, 0);

        WebLogin {
            webview,
            page,
            info_bar,
            info_bar_label,
            error_details_button,
            redirect_signal_id: gtk_handle!(None),
            info_bar_close_signal: None,
            info_bar_response_signal: None,
            error_details_signal: None,
        }
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }

    fn hide_info_bar(info_bar: &gtk::InfoBar) {
        info_bar.set_revealed(false);
        let clone = info_bar.clone();
        gtk::timeout_add(200, move || {
            clone.set_visible(false);
            gtk::Continue(false)
        });
    }

    pub fn show_error(&mut self, error: NewsFlashError) {
        GtkUtil::disconnect_signal(self.error_details_signal, &self.error_details_button);
        self.error_details_signal = None;

        match error.kind() {
            NewsFlashErrorKind::Login => self.info_bar_label.set_text("Failed to log in"),
            _ => self.info_bar_label.set_text("Unknown error."),
        }

        self.error_details_signal = Some(
            self.error_details_button
                .connect_clicked(move |button| {
                    let parent = GtkUtil::get_main_window(button).unwrap();
                    let _dialog = ErrorDialog::new(&error, &parent);
                })
                .to_glib(),
        );

        self.info_bar.set_visible(true);
        self.info_bar.set_revealed(true);
    }

    pub fn set_service(&mut self, info: PluginInfo) -> Result<(), Error> {
        // setup infobar
        self.info_bar_close_signal = Some(
            self.info_bar
                .connect_close(|info_bar| {
                    WebLogin::hide_info_bar(info_bar);
                })
                .to_glib(),
        );
        self.info_bar_response_signal = Some(
            self.info_bar
                .connect_response(|info_bar, response| {
                    if let ResponseType::Close = response {
                        WebLogin::hide_info_bar(info_bar);
                    }
                })
                .to_glib(),
        );

        if let LoginGUI::OAuth(web_login_desc) = info.login_gui.clone() {
            if let Some(url) = web_login_desc.clone().login_website {
                self.webview.load_uri(url.as_str());
                let redirect_signal_id = self.redirect_signal_id.clone();
                let signal_id = self.webview.connect_load_changed(move |webview, event| {
                    match event {
                        LoadEvent::Started | LoadEvent::Redirected => {
                            if let Some(redirect_url) = &web_login_desc.catch_redirect {
                                if let Some(uri) = webview.get_uri() {
                                    if uri.len() > redirect_url.len() && &uri[..redirect_url.len()] == redirect_url {
                                        let oauth_data = OAuthData {
                                            id: info.id.clone(),
                                            url: uri.as_str().to_owned(),
                                        };
                                        let oauth_data = LoginData::OAuth(oauth_data);
                                        let oauth_data_json = serde_json::to_string(&oauth_data).unwrap();
                                        if let Ok(main_window) = GtkUtil::get_main_window(webview) {
                                            if let Some(action) = main_window.lookup_action("login") {
                                                let login_data_json = Variant::from(&oauth_data_json);
                                                if let Some(signal_id) = *redirect_signal_id.borrow() {
                                                    let signal_id = SignalHandlerId::from_glib(signal_id);
                                                    webview.disconnect(signal_id);
                                                }
                                                webview.stop_loading();
                                                action.activate(Some(&login_data_json));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // do nothing
                        }
                    }
                });

                *self.redirect_signal_id.borrow_mut() = Some(signal_id.to_glib());
            }
        }

        Ok(())
    }

    pub fn reset(&self) {
        self.info_bar.set_revealed(false);
        self.info_bar.set_visible(false);
        GtkUtil::disconnect_signal(self.info_bar_close_signal, &self.info_bar);
        GtkUtil::disconnect_signal(self.info_bar_response_signal, &self.info_bar);
        self.webview.load_plain_text("");
    }
}
