use crate::content_page::{ContentHeader, ContentPage};
use crate::gtk_handle;
use crate::login_screen::{LoginHeaderbar, PasswordLogin, WebLogin};
use crate::main_window_actions::MainWindowActions;
use crate::main_window_state::MainWindowState;
use crate::welcome_screen::{WelcomeHeaderbar, WelcomePage};
use crate::Resources;
use failure::format_err;
use failure::Error;
use gtk::{self, Application, ApplicationWindow, Builder, CssProvider, CssProviderExt, GtkWindowExt, GtkWindowExtManual, Inhibit, Stack, StackExt, StyleContext, WidgetExt};
use log::{info, warn};
use news_flash::models::{Tag, TagID};
use news_flash::NewsFlash;
use std::cell::RefCell;
use std::rc::Rc;
use std::str;

pub static DATA_DIR: &'static str = ".news-flash";
const PANED_DEFAULT_POS: i32 = 600;

pub struct MainWindow {
    widget: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application) -> Result<Self, Error> {
        // setup CSS for window
        let provider = CssProvider::new();
        let css_data = Resources::get("css/app.css").ok_or_else(|| format_err!("some err"))?;
        CssProvider::load_from_data(&provider, css_data.as_ref()).unwrap();
        StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &provider, 600);

        let ui_data = Resources::get("ui/main_window.ui").ok_or_else(|| format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let window: ApplicationWindow = builder.get_object("main_window").ok_or_else(|| format_err!("some err"))?;
        let stack: Stack = builder.get_object("main_stack").ok_or_else(|| format_err!("some err"))?;

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

        let pw_login_handle = gtk_handle!(pw_login);
        let oauht_login_handle = gtk_handle!(oauth_login);
        let content_page_handle = gtk_handle!(content);
        let content_header_handle = gtk_handle!(content_header);
        let news_flash_handle = gtk_handle!(None);

        let state = gtk_handle!(MainWindowState::new());

        MainWindowActions::setup_show_password_page_action(&window, &pw_login_handle, &stack, login_header.widget());
        MainWindowActions::setup_show_oauth_page_action(&window, &oauht_login_handle, &stack, login_header.widget());
        MainWindowActions::setup_show_welcome_page_action(&window, &oauht_login_handle, &pw_login_handle, &stack, welcome_header.widget());
        MainWindowActions::setup_show_content_page_action(&window, &news_flash_handle, &stack, &content_page_handle, content_header_handle.borrow().widget());
        MainWindowActions::setup_login_action(&window, &news_flash_handle, &oauht_login_handle, &pw_login_handle);
        MainWindowActions::setup_sync_paned_action(&window, &content_page_handle, &content_header_handle);
        MainWindowActions::setup_sync_action(&window, &content_page_handle, &content_header_handle, &news_flash_handle, &state);
        MainWindowActions::setup_sidebar_selection_action(&window, &state);
        MainWindowActions::setup_headerbar_selection_action(&window, &state);
        MainWindowActions::setup_search_action(&window, &state);
        MainWindowActions::setup_update_article_list_action(&window, &state, &content_page_handle, &news_flash_handle);
        MainWindowActions::setup_show_more_articles_action(&window, &state, &content_page_handle, &news_flash_handle);
        MainWindowActions::setup_show_article_action(&window, &content_page_handle, &news_flash_handle);

        let mut data_dir = dirs::home_dir().expect("$HOME not available");
        data_dir.push(DATA_DIR);
        if let Ok(news_flash_lib) = NewsFlash::try_load(&data_dir) {
            info!("Successful load from config");

            stack.set_visible_child_name("content");
            let id = news_flash_lib.id().ok_or_else(|| format_err!("some err"))?;
            content_page_handle.borrow().set_service(&id, news_flash_lib.user_name())?;
            *news_flash_handle.borrow_mut() = Some(news_flash_lib);

            // try to fill content page with data
            content_page_handle.borrow_mut().update_sidebar(&news_flash_handle);
            content_page_handle.borrow_mut().update_article_list(&news_flash_handle, &state);

            window.set_titlebar(&content_header_handle.borrow().widget());
        } else {
            warn!("No account configured");
            stack.set_visible_child_name("welcome");
            window.set_titlebar(&welcome_header.widget());
        }

        content_header_handle.borrow().set_paned(PANED_DEFAULT_POS);
        content_page_handle.borrow().set_paned(PANED_DEFAULT_POS);
        window.show_all();

        let main_window = MainWindow { widget: window };

        Ok(main_window)
    }

    pub fn present(&self) {
        self.widget.present();
    }

    pub fn demo_tag_list() -> Vec<Tag> {
        let tag_1 = Tag {
            tag_id: TagID::new("Tag_1"),
            label: "Tag Label 1".to_owned(),
            color: Some("#4696C8".to_owned()),
            sort_index: Some(0),
        };
        let tag_2 = Tag {
            tag_id: TagID::new("Tag_2"),
            label: "Tag Label 2".to_owned(),
            color: Some("#FF0000".to_owned()),
            sort_index: Some(1),
        };
        let tag_3 = Tag {
            tag_id: TagID::new("Tag_3"),
            label: "Tag Label 3".to_owned(),
            color: Some("#2565FA".to_owned()),
            sort_index: Some(2),
        };
        let mut list = Vec::new();
        list.push(tag_1);
        list.push(tag_2);
        list.push(tag_3);
        list
    }
}
