use gio::{
    ActionMapExt,
    ActionExt,
};
use glib::{
    Variant,
    signal::SignalHandlerId,
    translate::{
        ToGlib,
        FromGlib,
    },
};
use webkit2gtk::{
    WebContext,
    WebView,
    WebViewExt,
    WebViewExtManual,
    UserContentManager,
    LoadEvent,
};
use gtk::{
    ObjectExt,
};
use failure::Error;
use failure::format_err;
use news_flash::models::{
    PluginInfo,
    LoginGUI,
    LoginData,
    OAuthData,
};
use crate::gtk_util::GtkUtil;
use crate::main_window::GtkHandle;
use std::rc::Rc;
use std::cell::RefCell;


#[derive(Clone, Debug)]
pub struct WebLogin {
    webview: WebView,
    redirect_signal_id: GtkHandle<Option<u64>>,
}

impl WebLogin {
    pub fn new() -> Result<Self, Error> {
        
        let context = WebContext::get_default().ok_or(format_err!("some err"))?;
        let content_manager = UserContentManager::new();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &content_manager);

        let page = WebLogin {
            webview: webview,
            redirect_signal_id: Rc::new(RefCell::new(None)),
        };

        Ok(page)
    }

    pub fn widget(&self) -> WebView {
        self.webview.clone()
    }

    pub fn set_service(&self, info: PluginInfo) -> Result<(), Error> {

        if let LoginGUI::OAuth(web_login_desc) = info.login_gui.clone() {
            if let Some(url) = web_login_desc.clone().login_website {
                self.webview.load_uri(url.as_str());
                let redirect_signal_id = self.redirect_signal_id.clone();
                let signal_id = self.webview.connect_load_changed(move |webview, event| {
                    match event {
                        LoadEvent::Started |
                        LoadEvent::Redirected => {
                            if let Some(redirect_url) = &web_login_desc.catch_redirect {
                                if let Some(uri) = webview.get_uri() {
                                    if uri.len() > redirect_url.len()
                                    && &uri[..redirect_url.len()] == redirect_url {
                                        let oauth_data = OAuthData {
                                            id: info.id.clone(),
                                            url: uri,
                                        };
                                        let oauth_data = LoginData::OAuth(oauth_data);
                                        let oauth_data_json = serde_json::to_string(&oauth_data).unwrap();
                                        if let Ok(main_window) = GtkUtil::get_main_window(webview) {
                                            if let Some(action) = main_window.lookup_action("login") {
                                                let login_data_json = Variant::from(&oauth_data_json);
                                                // FIXME: stop action activating multiple times
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
                        },
                        _ => {
                            // do nothing
                        },
                    }
                });

                *self.redirect_signal_id.borrow_mut() = Some(signal_id.to_glib());
            }
        }

        Ok(())
    }

    pub fn reset(&self) {
        self.webview.load_plain_text("");
    }
}