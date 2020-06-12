use super::error::{LoginScreenError, LoginScreenErrorKind};
use crate::app::Action;
use crate::error_dialog::ErrorDialog;
use crate::i18n::i18n;
use crate::util::{BuilderHelper, GtkUtil, Util, GTK_BUILDER_ERROR};
use glib::{clone, source::Continue, translate::ToGlib, Sender};
use gtk::{Box, BoxExt, Button, ButtonExt, InfoBar, InfoBarExt, Label, LabelExt, ResponseType, WidgetExt};
use news_flash::models::{LoginData, LoginGUI, OAuthData, PluginInfo};
use news_flash::{NewsFlashError, NewsFlashErrorKind};
use parking_lot::RwLock;
use std::rc::Rc;
use webkit2gtk::{LoadEvent, UserContentManager, WebContext, WebView, WebViewExt, WebViewExtManual};

#[derive(Debug)]
pub struct WebLogin {
    sender: Sender<Action>,
    webview: WebView,
    page: gtk::Box,
    info_bar: gtk::InfoBar,
    info_bar_label: gtk::Label,
    error_details_button: gtk::Button,
    redirect_signal_id: Rc<RwLock<Option<usize>>>,
    info_bar_close_signal: RwLock<Option<usize>>,
    info_bar_response_signal: RwLock<Option<usize>>,
    error_details_signal: RwLock<Option<usize>>,
}

impl WebLogin {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let page = builder.get::<Box>("oauth_box");
        let info_bar = builder.get::<InfoBar>("oauth_info_bar");
        let error_details_button = builder.get::<Button>("oauth_details_button");
        let info_bar_label = builder.get::<Label>("oauth_info_bar_label");

        let context = WebContext::get_default().expect(GTK_BUILDER_ERROR);
        let content_manager = UserContentManager::new();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &content_manager);

        page.pack_start(&webview, true, true, 0);

        WebLogin {
            sender,
            webview,
            page,
            info_bar,
            info_bar_label,
            error_details_button,
            redirect_signal_id: Rc::new(RwLock::new(None)),
            info_bar_close_signal: RwLock::new(None),
            info_bar_response_signal: RwLock::new(None),
            error_details_signal: RwLock::new(None),
        }
    }

    fn hide_info_bar(info_bar: &gtk::InfoBar) {
        info_bar.set_revealed(false);
        gtk::timeout_add(
            200,
            clone!(@weak info_bar => @default-panic, move || {
                info_bar.set_visible(false);
                Continue(false)
            }),
        );
    }

    pub fn show_error(&self, error: NewsFlashError) {
        GtkUtil::disconnect_signal(*self.error_details_signal.read(), &self.error_details_button);
        self.error_details_signal.write().take();

        match error.kind() {
            NewsFlashErrorKind::Login => self.info_bar_label.set_text(&i18n("Failed to log in")),
            _ => self.info_bar_label.set_text(&i18n("Unknown error.")),
        }

        self.error_details_button.show();
        self.error_details_signal.write().replace(
            self.error_details_button
                .connect_clicked(move |button| {
                    let parent = GtkUtil::get_main_window(button).expect("MainWindow is not parent of details button.");
                    let _dialog = ErrorDialog::new(&error, &parent);
                })
                .to_glib() as usize,
        );

        self.info_bar.set_visible(true);
        self.info_bar.set_revealed(true);
    }

    pub fn set_service(&self, info: PluginInfo) -> Result<(), LoginScreenError> {
        // setup infobar
        self.info_bar_close_signal.write().replace(
            self.info_bar
                .connect_close(|info_bar| {
                    WebLogin::hide_info_bar(info_bar);
                })
                .to_glib() as usize,
        );
        self.info_bar_response_signal.write().replace(
            self.info_bar
                .connect_response(|info_bar, response| {
                    if let ResponseType::Close = response {
                        WebLogin::hide_info_bar(info_bar);
                    }
                })
                .to_glib() as usize,
        );

        if let LoginGUI::OAuth(web_login_desc) = info.login_gui.clone() {
            if let Some(url) = web_login_desc.clone().login_website {
                self.webview.load_uri(url.as_str());
                let signal_id = self.webview.connect_load_changed(clone!(
                    @weak self.redirect_signal_id as redirect_signal_id,
                    @strong self.sender as sender => @default-panic, move |webview, event|
                {
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
                                        GtkUtil::disconnect_signal(*redirect_signal_id.read(), webview);
                                        webview.stop_loading();
                                        Util::send(&sender, Action::Login(oauth_data));
                                    }
                                }
                            }
                        }
                        _ => {
                            // do nothing
                        }
                    }
                }));

                self.redirect_signal_id.write().replace(signal_id.to_glib() as usize);
                return Ok(());
            }

            return Err(LoginScreenErrorKind::OauthUrl.into());
        }

        Err(LoginScreenErrorKind::LoginGUI.into())
    }

    pub fn reset(&self) {
        self.info_bar.set_revealed(false);
        self.info_bar.set_visible(false);
        GtkUtil::disconnect_signal(*self.info_bar_close_signal.read(), &self.info_bar);
        GtkUtil::disconnect_signal(*self.info_bar_response_signal.read(), &self.info_bar);
        self.webview.load_plain_text("");
    }
}
