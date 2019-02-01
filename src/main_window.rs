use gtk::{
    self,
    Application,
    ApplicationWindow,
    GtkWindowExt,
    WidgetExt,
    Inhibit,
    CssProvider,
    CssProviderExt,
    GtkWindowExtManual,
    StyleContext,
    Builder,
    Stack,
    StackExt,
    StackTransitionType,
    HeaderBar,
};
use crate::welcome_screen::{
    WelcomePage,
    WelcomeHeaderbar,
};
use crate::login_screen::{
    PasswordLogin,
    WebLogin,
    LoginHeaderbar,
};
use gio::{
    SimpleAction,
    SimpleActionExt,
    ActionMapExt,
    ActionExt,
};
use news_flash::models::{
    PluginID,
    LoginData,
};
use log::{
    info,
    warn,
    error,
};
use news_flash::NewsFlash;
use std::collections::HashMap;
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::content_page::{
    ContentPage,
    ContentHeader,
};

pub type GtkHandle<T> = Rc<RefCell<T>>;
pub type GtkHandleMap<T, K> = GtkHandle<HashMap<T, K>>;

static DATA_DIR: &'static str = "/home/jeanluc/.news-flash";

pub struct MainWindow {
    widget: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application) -> Result<Self, Error> {

        // setup CSS for window
        let provider = CssProvider::new();
        let css_data = Resources::get("css/app.css").ok_or(format_err!("some err"))?;
        CssProvider::load_from_data(&provider, css_data.as_ref()).unwrap();
        StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().unwrap(),
            &provider,
            600,
        );

        let ui_data = Resources::get("ui/main_window.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let window : ApplicationWindow = builder.get_object("main_window").ok_or(format_err!("some err"))?;
        let stack : Stack = builder.get_object("main_stack").ok_or(format_err!("some err"))?;

        let login_header = LoginHeaderbar::new(&window)?;
        let welcome_header = WelcomeHeaderbar::new()?;
        let content_header = ContentHeader::new()?;

        window.set_application(app);
        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        let welcome = Self::setup_welcome_page(&window)?;
        stack.add_named(&welcome.widget(), "welcome");

        let pw_login = Self::setup_password_login_page()?;
        stack.add_named(&pw_login.widget(), "password_login");

        let oauth_login = Self::setup_web_login_page()?;
        stack.add_named(&oauth_login.widget(), "oauth_login");

        let content = Self::setup_content_page()?;
        stack.add_named(&content.widget(), "content");
        
        let pw_login_handle = Rc::new(RefCell::new(pw_login));
        let oauht_login_handle = Rc::new(RefCell::new(oauth_login));
        let content_page_handle = Rc::new(RefCell::new(content));
        let news_flash_handle = Rc::new(RefCell::new(None));
        
        Self::setup_show_password_page_action(&window, &pw_login_handle, &stack, login_header.widget());
        Self::setup_show_oauth_page_action(&window, &oauht_login_handle, &stack, login_header.widget());
        Self::setup_show_welcome_page_action(&window, &oauht_login_handle, &pw_login_handle, &stack, welcome_header.widget());
        Self::setup_show_content_page_action(&window, &news_flash_handle, &stack, &content_page_handle, content_header.widget());
        Self::setup_login_action(&window, &news_flash_handle, &oauht_login_handle, &pw_login_handle);

        if let Ok(news_flash_lib) = NewsFlash::try_load(&PathBuf::from(DATA_DIR)) {
            info!("Successful load from config");

            stack.set_visible_child_name("content");
            let id = news_flash_lib.id().ok_or(format_err!("some err"))?;
            content_page_handle.borrow().set_service(&id, news_flash_lib.user_name())?;

            *news_flash_handle.borrow_mut() = Some(news_flash_lib);
            window.set_titlebar(&content_header.widget());
        }
        else {
            warn!("No account configured");
            stack.set_visible_child_name("welcome");
            window.set_titlebar(&welcome_header.widget());
        }
        window.show_all();

        let main_window = MainWindow {
            widget: window,
        };

        Ok(main_window)
    }

    fn setup_welcome_page(window: &ApplicationWindow) -> Result<WelcomePage, Error> {
        let welcome = WelcomePage::new(window)?;
        Ok(welcome)
    }

    fn setup_password_login_page() -> Result<PasswordLogin, Error> {
        let pw_login = PasswordLogin::new()?;
        Ok(pw_login)
    }

    fn setup_web_login_page() -> Result<WebLogin, Error> {
        let web_login = WebLogin::new()?;
        Ok(web_login)
    }

    fn setup_content_page() -> Result<ContentPage, Error> {
        let content = ContentPage::new()?;
        Ok(content)
    }

    fn setup_show_password_page_action(window: &ApplicationWindow, pw_page: &GtkHandle<PasswordLogin>, stack: &Stack, headerbar: HeaderBar) {
        let application_window = window.clone();
        let stack = stack.clone();
        let show_pw_page = SimpleAction::new("show-pw-page", glib::VariantTy::new("s").ok());
        let pw_page = pw_page.clone();
        show_pw_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    if let Some(service_meta) = NewsFlash::list_backends().get(&id) {
                        if let Ok(()) = pw_page.borrow_mut().set_service(service_meta.clone()) {
                            application_window.set_titlebar(&headerbar);
                            stack.set_transition_type(StackTransitionType::SlideLeft);
                            stack.set_visible_child_name("password_login");
                        }
                    }
                }
            }
        });
        show_pw_page.set_enabled(true);
        window.add_action(&show_pw_page);
    }

    fn setup_show_oauth_page_action(window: &ApplicationWindow, oauth_page: &GtkHandle<WebLogin>, stack: &Stack, headerbar: HeaderBar) {
        let application_window = window.clone();
        let stack = stack.clone();
        let oauth_page = oauth_page.clone();
        let show_pw_page = SimpleAction::new("show-oauth-page", glib::VariantTy::new("s").ok());
        show_pw_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    if let Some(service_meta) = NewsFlash::list_backends().get(&id) {
                        if let Ok(()) = oauth_page.borrow_mut().set_service(service_meta.clone()) {
                            application_window.set_titlebar(&headerbar);
                            stack.set_transition_type(StackTransitionType::SlideLeft);
                            stack.set_visible_child_name("oauth_login");
                        }
                    }
                }
            }
        });
        show_pw_page.set_enabled(true);
        window.add_action(&show_pw_page);
    }

    fn setup_show_welcome_page_action(
        window: &ApplicationWindow,
        oauth_page: &GtkHandle<WebLogin>,
        pw_page: &GtkHandle<PasswordLogin>,
        stack: &Stack,
        headerbar: HeaderBar
    ) {
        let application_window = window.clone();
        let stack = stack.clone();
        let show_welcome_page = SimpleAction::new("show-welcome-page", None);
        let pw_page = pw_page.clone();
        let oauth_page = oauth_page.clone();
        show_welcome_page.connect_activate(move |_action, _data| {
            application_window.set_titlebar(&headerbar);
            pw_page.borrow_mut().reset();
            oauth_page.borrow_mut().reset();
            stack.set_transition_type(StackTransitionType::SlideRight);
            stack.set_visible_child_name("welcome");
        });
        show_welcome_page.set_enabled(true);
        window.add_action(&show_welcome_page);
    }

    fn setup_show_content_page_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        stack: &Stack,
        content_page: &GtkHandle<ContentPage>,
        headerbar: gtk::Paned
    ) {
        let news_flash = news_flash.clone();
        let application_window = window.clone();
        let stack = stack.clone();
        let content_page = content_page.clone();
        let show_content_page = SimpleAction::new("show-content-page", glib::VariantTy::new("s").ok());
        show_content_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    let mut user_name : Option<String> = None;
                    if let Some(api) = &*news_flash.borrow() {
                        user_name = api.user_name();
                    }
                    content_page.borrow().set_service(&id, user_name).unwrap();
                    application_window.set_titlebar(&headerbar);
                    stack.set_transition_type(StackTransitionType::SlideLeft);
                    stack.set_visible_child_name("content");
                }
            }
        });
        show_content_page.set_enabled(true);
        window.add_action(&show_content_page);
    }

    fn setup_login_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        oauth_page: &GtkHandle<WebLogin>,
        pw_page: &GtkHandle<PasswordLogin>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let pw_page = pw_page.clone();
        let oauth_page = oauth_page.clone();
        let login_action = SimpleAction::new("login", glib::VariantTy::new("s").ok());
        login_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let info: LoginData = serde_json::from_str(&data).unwrap();
                    let id = match &info {
                        LoginData::OAuth(oauth) => oauth.id.clone(),
                        LoginData::Password(pass) => pass.id.clone(),
                    };
                    let mut news_flash_lib = NewsFlash::new(&PathBuf::from(DATA_DIR), &id).unwrap();
                    match news_flash_lib.login(info.clone()) {
                        Ok(()) => {
                            // create main obj
                            *news_flash.borrow_mut() = Some(news_flash_lib);

                            // show content page
                            if let Some(action) = main_window.lookup_action("show-content-page") {
                                let id = glib::Variant::from(id.to_str());
                                action.activate(Some(&id));
                            }
                        },
                        Err(error) => {
                            error!("Login failed! Plguin: {}, Error: {}", id, error);
                            match info {
                                LoginData::OAuth(_) => {
                                    oauth_page.borrow_mut().show_error(error);
                                },
                                LoginData::Password(_) => {
                                    pw_page.borrow_mut().show_error(error);
                                },
                            }
                        },
                    }
                }
            }
        });
        login_action.set_enabled(true);
        window.add_action(&login_action);
    }

    pub fn present(&self) {
        self.widget.present();
    }
}