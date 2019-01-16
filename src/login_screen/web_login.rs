use gtk::{
    self,
};
use webkit2gtk::{
    SettingsExt,
    WebContext,
    WebContextExt,
    WebView,
    WebViewExt,
    WebViewExtManual,
    UserContentManager,
};
use failure::Error;
use failure::format_err;
use news_flash::models::{
    PluginMetadata,
    LoginGUI,
};


#[derive(Clone, Debug)]
pub struct WebLogin {
    webview: WebView,
}

impl WebLogin {
    pub fn new() -> Result<Self, Error> {
        
        let context = WebContext::get_default().ok_or(format_err!("some err"))?;
        let content_manager = UserContentManager::new();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &content_manager);

        let page = WebLogin {
            webview: webview,
        };

        Ok(page)
    }

    pub fn widget(&self) -> WebView {
        self.webview.clone()
    }

    pub fn set_service(&self, _info: PluginMetadata, gui_desc: LoginGUI) -> Result<(), Error> {

        if let LoginGUI::OAuth(web_login_desc) = gui_desc {
            if let Some(url) = web_login_desc.login_website {
                self.webview.load_uri(url.as_str());
            }
        }

        Ok(())
    }

    pub fn reset(&self) {
        self.webview.load_plain_text("");
    }
}