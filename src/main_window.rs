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
use crate::sidebar::{
    FeedListTree,
};
use gio::{
    SimpleAction,
    SimpleActionExt,
    ActionMapExt,
    ActionExt,
};
use news_flash::models::{
    Category as CategoryModel,
    Feed as FeedModel,
    FeedMapping,
    CategoryID,
    FeedID,
    PluginID,
    LoginData,
    NEWSFLASH_TOPLEVEL,
};
use log::{
    info,
    warn,
    error,
};
use news_flash::NewsFlash;
use crate::error_dialog::ErrorDialog;
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
const PANED_DEFAULT_POS: i32 = 600;

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
        let content_header_handle = Rc::new(RefCell::new(content_header));
        let news_flash_handle = Rc::new(RefCell::new(None));
        
        Self::setup_show_password_page_action(&window, &pw_login_handle, &stack, login_header.widget());
        Self::setup_show_oauth_page_action(&window, &oauht_login_handle, &stack, login_header.widget());
        Self::setup_show_welcome_page_action(&window, &oauht_login_handle, &pw_login_handle, &stack, welcome_header.widget());
        Self::setup_show_content_page_action(&window, &news_flash_handle, &stack, &content_page_handle, content_header_handle.borrow().widget());
        Self::setup_login_action(&window, &news_flash_handle, &oauht_login_handle, &pw_login_handle);
        Self::setup_sync_paned_action(&window, &content_page_handle, &content_header_handle);
        Self::setup_sync_action(&window, &content_page_handle, &content_header_handle, &news_flash_handle);

        if let Ok(news_flash_lib) = NewsFlash::try_load(&PathBuf::from(DATA_DIR)) {
            info!("Successful load from config");

            stack.set_visible_child_name("content");
            let id = news_flash_lib.id().ok_or(format_err!("some err"))?;
            content_page_handle.borrow().set_service(&id, news_flash_lib.user_name())?;

            // try to fill content page with data
            let mut tree = FeedListTree::new();
            let categories = news_flash_lib.get_categories().unwrap();
            for category in categories {
                tree.add_category(&category, 0).unwrap();
            }
            content_page_handle.borrow_mut().update_feedlist(tree);

            *news_flash_handle.borrow_mut() = Some(news_flash_lib);
            window.set_titlebar(&content_header_handle.borrow().widget());
        }
        else {
            warn!("No account configured");
            stack.set_visible_child_name("welcome");
            window.set_titlebar(&welcome_header.widget());
        }

        content_header_handle.borrow().set_paned(PANED_DEFAULT_POS);
        content_page_handle.borrow().set_paned(PANED_DEFAULT_POS);
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

    fn setup_sync_paned_action(
        window: &ApplicationWindow,
        content_page: &GtkHandle<ContentPage>,
        content_header: &GtkHandle<ContentHeader>,
    ) {
        let content_page = content_page.clone();
        let content_header = content_header.clone();
        let sync_paned = SimpleAction::new("sync-paned", glib::VariantTy::new("i").ok());
        sync_paned.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(pos) = data.get::<i32>() {
                    content_page.borrow().set_paned(pos);
                    content_header.borrow().set_paned(pos);
                }
            }
        });
        sync_paned.set_enabled(true);
        window.add_action(&sync_paned);
    }

    fn setup_sync_action(
        window: &ApplicationWindow,
        content_page: &GtkHandle<ContentPage>,
        content_header: &GtkHandle<ContentHeader>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let _content_page = content_page.clone();
        let parent = window.clone();
        let content_header = content_header.clone();
        let news_flash = news_flash.clone();
        let sync_action = SimpleAction::new("sync", None);
        sync_action.connect_activate(move |_action, _data| {
            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                match news_flash.sync() {
                    Ok(()) => content_header.borrow().finish_sync(),
                    Err(error) => {
                        let _dialog = ErrorDialog::new(&error, &parent).unwrap();
                    },
                }
            }
        });
        sync_action.set_enabled(true);
        window.add_action(&sync_action);
    }

    pub fn present(&self) {
        self.widget.present();
    }

    fn demo_feedlist() -> FeedListTree {

        let category_1 = CategoryModel {
            category_id: CategoryID::new("category_1"),
            label: "category 1".to_owned(),
            sort_index: None,
            parent_id: NEWSFLASH_TOPLEVEL.clone(),
        };
        let feed_1 = FeedModel {
            feed_id: FeedID::new("feed_1"),
            label: "Feed 1".to_owned(),
            website: None,
            feed_url: None,
            icon_url: None,
            sort_index: Some(2),
        };
        let mapping_1 = FeedMapping {
            feed_id: FeedID::new("feed_1"),
            category_id: CategoryID::new("category_1"),
        };
        let feed_2 = FeedModel {
            feed_id: FeedID::new("feed_2"),
            label: "Feed 2".to_owned(),
            website: None,
            feed_url: None,
            icon_url: None,
            sort_index: Some(1),
        };
        let mapping_2 = FeedMapping {
            feed_id: FeedID::new("feed_2"),
            category_id: CategoryID::new("category_1"),
        };
        let category_2 = CategoryModel {
            category_id: CategoryID::new("category_2"),
            label: "category 2".to_owned(),
            sort_index: Some(0),
            parent_id: CategoryID::new("category_1"),
        };
        let feed_3 = FeedModel {
            feed_id: FeedID::new("feed_3"),
            label: "Feed 3".to_owned(),
            website: None,
            feed_url: None,
            icon_url: None,
            sort_index: Some(0),
        };
        let mapping_3 = FeedMapping {
            feed_id: FeedID::new("feed_3"),
            category_id: CategoryID::new("category_2"),
        };

        
        let mut tree = FeedListTree::new();
        tree.add_category(&category_1, 7).unwrap();
        tree.add_category(&category_2, 5).unwrap();
        tree.add_feed(&feed_1, &mapping_1, 2).unwrap();
        tree.add_feed(&feed_2, &mapping_2, 0).unwrap();
        tree.add_feed(&feed_3, &mapping_3, 5).unwrap();
        tree
    }
}