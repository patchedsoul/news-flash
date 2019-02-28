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
    TagListModel,
};
use log::{
    info,
    warn,
};
use news_flash::NewsFlash;
use news_flash::models::{
    Tag,
    TagID,
};
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
use crate::main_window_actions::MainWindowActions;

pub type GtkHandle<T> = Rc<RefCell<T>>;
pub type GtkHandleMap<T, K> = GtkHandle<HashMap<T, K>>;

pub static DATA_DIR: &'static str = "/home/jeanluc/.news-flash";
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

        // setup pages
        let welcome = WelcomePage::new(&window)?;
        stack.add_named(&welcome.widget(), "welcome");

        let pw_login = PasswordLogin::new()?;
        stack.add_named(&pw_login.widget(), "password_login");

        let oauth_login = WebLogin::new()?;
        stack.add_named(&oauth_login.widget(), "oauth_login");

        let content = ContentPage::new()?;
        stack.add_named(&content.widget(), "content");
        
        let pw_login_handle = Rc::new(RefCell::new(pw_login));
        let oauht_login_handle = Rc::new(RefCell::new(oauth_login));
        let content_page_handle = Rc::new(RefCell::new(content));
        let content_header_handle = Rc::new(RefCell::new(content_header));
        let news_flash_handle = Rc::new(RefCell::new(None));
        
        MainWindowActions::setup_show_password_page_action(&window, &pw_login_handle, &stack, login_header.widget());
        MainWindowActions::setup_show_oauth_page_action(&window, &oauht_login_handle, &stack, login_header.widget());
        MainWindowActions::setup_show_welcome_page_action(&window, &oauht_login_handle, &pw_login_handle, &stack, welcome_header.widget());
        MainWindowActions::setup_show_content_page_action(&window, &news_flash_handle, &stack, &content_page_handle, content_header_handle.borrow().widget());
        MainWindowActions::setup_login_action(&window, &news_flash_handle, &oauht_login_handle, &pw_login_handle);
        MainWindowActions::setup_sync_paned_action(&window, &content_page_handle, &content_header_handle);
        MainWindowActions::setup_sync_action(&window, &content_page_handle, &content_header_handle, &news_flash_handle);

        if let Ok(news_flash_lib) = NewsFlash::try_load(&PathBuf::from(DATA_DIR)) {
            info!("Successful load from config");

            stack.set_visible_child_name("content");
            let id = news_flash_lib.id().ok_or(format_err!("some err"))?;
            content_page_handle.borrow().set_service(&id, news_flash_lib.user_name())?;
            *news_flash_handle.borrow_mut() = Some(news_flash_lib);

            // try to fill content page with data
            Self::update_sidebar(&news_flash_handle, &content_page_handle);

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

    pub fn present(&self) {
        self.widget.present();
    }

    pub fn update_sidebar(
        news_flash_handle: &GtkHandle<Option<NewsFlash>>,
        content_page_handle: &GtkHandle<ContentPage>,
    ) {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            Self::update_sidebar_from_ref(news_flash, content_page_handle);
        }
    }

    pub fn update_sidebar_from_ref(
        news_flash: &mut NewsFlash,
        content_page_handle: &GtkHandle<ContentPage>,
    ) {
        // feedlist
        let mut tree = FeedListTree::new();
        let categories = news_flash.get_categories().unwrap();
        for category in categories {
            let count = news_flash.unread_count_category(&category.category_id).unwrap();
            tree.add_category(&category, count as i32).unwrap();
        }
        let (feeds, mappings) = news_flash.get_feeds().unwrap();
        for mapping in mappings {
            let count = news_flash.unread_count_feed(&mapping.feed_id).unwrap();
            let feed = feeds.iter().find(|feed| feed.feed_id == mapping.feed_id).unwrap();
            let favicon = match news_flash.get_icon_info(&feed) {
                Ok(favicon) => Some(favicon),
                Err(_) => None,
            };
            tree.add_feed(&feed, &mapping, count as i32, favicon).unwrap();
        }

        // tag list
        let mut list = TagListModel::new();
        //let tags = news_flash.get_tags().unwrap();
        let tags = Self::demo_tag_list();
        for tag in tags {
            let count = news_flash.unread_count_tags(&tag.tag_id).unwrap();
            list.add(&tag, count as i32).unwrap();
        }
        
        let total_unread = news_flash.unread_count_all().unwrap();
        content_page_handle.borrow_mut().update_sidebar(tree, list, total_unread);
    }

    fn demo_tag_list() -> Vec<Tag> {
        let tag_1 = Tag {
            tag_id: TagID::new("Tag_1"),
            label: "label_1".to_owned(),
            color: None,
            sort_index: Some(0),
        };
        let tag_2 = Tag {
            tag_id: TagID::new("Tag_2"),
            label: "label_2".to_owned(),
            color: None,
            sort_index: Some(1),
        };
        let tag_3 = Tag {
            tag_id: TagID::new("Tag_3"),
            label: "label_3".to_owned(),
            color: None,
            sort_index: Some(2),
        };
        let mut list = Vec::new();
        list.push(tag_1);
        list.push(tag_2);
        list.push(tag_3);
        list
    }
}