use crate::article_list::{ReadUpdate, MarkUpdate};
use crate::content_page::HeaderSelection;
use crate::content_page::{ContentHeader, ContentPage};
use crate::error_dialog::ErrorDialog;
use crate::login_screen::{PasswordLogin, WebLogin};
use crate::main_window::DATA_DIR;
use crate::main_window_state::MainWindowState;
use crate::sidebar::models::SidebarSelection;
use crate::util::GtkHandle;
use gio::{ActionExt, ActionMapExt, SimpleAction};
use gtk::{self, ApplicationWindow, GtkWindowExt, HeaderBar, Stack, StackExt, StackTransitionType};
use log::error;
use news_flash::models::{ArticleID, LoginData, PluginID};
use news_flash::{NewsFlash, NewsFlashError};
use std::path::PathBuf;

pub struct MainWindowActions;

impl MainWindowActions {
    pub fn setup_show_password_page_action(
        window: &ApplicationWindow,
        pw_page: &GtkHandle<PasswordLogin>,
        stack: &Stack,
        headerbar: HeaderBar,
    ) {
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

    pub fn setup_show_oauth_page_action(
        window: &ApplicationWindow,
        oauth_page: &GtkHandle<WebLogin>,
        stack: &Stack,
        headerbar: HeaderBar,
    ) {
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

    pub fn setup_show_welcome_page_action(
        window: &ApplicationWindow,
        oauth_page: &GtkHandle<WebLogin>,
        pw_page: &GtkHandle<PasswordLogin>,
        stack: &Stack,
        headerbar: HeaderBar,
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

    pub fn setup_show_content_page_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        stack: &Stack,
        content_page: &GtkHandle<ContentPage>,
        headerbar: gtk::Paned,
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
                    let mut user_name: Option<String> = None;
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

    pub fn setup_login_action(
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
                        }
                        Err(error) => {
                            error!("Login failed! Plguin: {}, Error: {}", id, error);
                            match info {
                                LoginData::OAuth(_) => {
                                    oauth_page.borrow_mut().show_error(error);
                                }
                                LoginData::Password(_) => {
                                    pw_page.borrow_mut().show_error(error);
                                }
                            }
                        }
                    }
                }
            }
        });
        login_action.set_enabled(true);
        window.add_action(&login_action);
    }

    pub fn setup_sync_paned_action(
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

    pub fn setup_sync_action(
        window: &ApplicationWindow,
        content_header: &GtkHandle<ContentHeader>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let parent = window.clone();
        let content_header = content_header.clone();
        let news_flash = news_flash.clone();
        let sync_action = SimpleAction::new("sync", None);
        sync_action.connect_activate(move |_action, _data| {
            let mut result : Result<(), NewsFlashError> = Ok(());
            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                result = news_flash.sync();
            }
            match result {
                Ok(()) => {
                    content_header.borrow().finish_sync();
                    if let Some(action) = parent.lookup_action("update-sidebar") {
                        action.activate(None);
                    }
                    if let Some(action) = parent.lookup_action("update-article-list") {
                        action.activate(None);
                    }
                }
                Err(error) => {
                    let _dialog = ErrorDialog::new(&error, &parent);
                }
            }
        });
        sync_action.set_enabled(true);
        window.add_action(&sync_action);
    }

    pub fn setup_update_sidebar_action(
        window: &ApplicationWindow,
        content_page: &GtkHandle<ContentPage>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        state: &GtkHandle<MainWindowState>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let sync_action = SimpleAction::new("update-sidebar", None);
        sync_action.connect_activate(move |_action, _data| {
            content_page.borrow_mut().update_sidebar(&news_flash, &state);
        });
        sync_action.set_enabled(true);
        window.add_action(&sync_action);
    }

    pub fn setup_sidebar_selection_action(window: &ApplicationWindow, state: &GtkHandle<MainWindowState>) {
        let state = state.clone();
        let main_window = window.clone();
        let sidebar_selection_action = SimpleAction::new("sidebar-selection", glib::VariantTy::new("s").ok());
        sidebar_selection_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let selection: SidebarSelection = serde_json::from_str(&data).unwrap();
                    state.borrow_mut().set_sidebar_selection(selection);
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
                }
            }
        });
        sidebar_selection_action.set_enabled(true);
        window.add_action(&sidebar_selection_action);
    }

    pub fn setup_headerbar_selection_action(window: &ApplicationWindow, state: &GtkHandle<MainWindowState>) {
        let state = state.clone();
        let main_window = window.clone();
        let headerbar_selection_action = SimpleAction::new("headerbar-selection", glib::VariantTy::new("s").ok());
        headerbar_selection_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let new_selection: HeaderSelection = serde_json::from_str(&data).unwrap();
                    let old_selection = state.borrow().get_header_selection().clone();
                    state.borrow_mut().set_header_selection(new_selection.clone());
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
                    let update_sidebar = match old_selection {
                        HeaderSelection::All | HeaderSelection::Unread => {
                            match new_selection {
                                HeaderSelection::All | HeaderSelection::Unread => false,
                                HeaderSelection::Marked => true,
                            }
                        },
                        HeaderSelection::Marked => {
                            match new_selection {
                                HeaderSelection::All | HeaderSelection::Unread => true,
                                HeaderSelection::Marked => false,
                            }
                        },
                    };
                    if update_sidebar {
                        if let Some(action) = main_window.lookup_action("update-sidebar") {
                            action.activate(None);
                        }
                    }
                }
            }
        });
        headerbar_selection_action.set_enabled(true);
        window.add_action(&headerbar_selection_action);
    }

    pub fn setup_search_action(window: &ApplicationWindow, state: &GtkHandle<MainWindowState>) {
        let state = state.clone();
        let main_window = window.clone();
        let search_action = SimpleAction::new("search-term", glib::VariantTy::new("s").ok());
        search_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    state.borrow_mut().set_search_term(Some(data.to_owned()));
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
                }
            }
        });
        search_action.set_enabled(true);
        window.add_action(&search_action);
    }

    pub fn setup_update_article_list_action(
        window: &ApplicationWindow,
        state: &GtkHandle<MainWindowState>,
        content_page: &GtkHandle<ContentPage>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let update_article_list_action = SimpleAction::new("update-article-list", None);
        update_article_list_action.connect_activate(move |_action, _data| {
            content_page.borrow_mut().update_article_list(&news_flash, &state);
        });
        update_article_list_action.set_enabled(true);
        window.add_action(&update_article_list_action);
    }

    pub fn setup_show_more_articles_action(
        window: &ApplicationWindow,
        state: &GtkHandle<MainWindowState>,
        content_page: &GtkHandle<ContentPage>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let show_more_articles_action = SimpleAction::new("show-more-articles", None);
        show_more_articles_action.connect_activate(move |_action, _data| {
            content_page
                .borrow_mut()
                .load_more_articles(&news_flash, &state)
                .unwrap();
        });
        show_more_articles_action.set_enabled(true);
        window.add_action(&show_more_articles_action);
    }

    pub fn setup_show_article_action(
        window: &ApplicationWindow,
        content_page: &GtkHandle<ContentPage>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let show_article_action = SimpleAction::new("show-article", glib::VariantTy::new("s").ok());
        show_article_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let article_id = ArticleID::new(data);
                    content_page
                        .borrow_mut()
                        .show_article(&article_id, &news_flash)
                        .unwrap();
                }
            }
        });
        show_article_action.set_enabled(true);
        window.add_action(&show_article_action);
    }

    pub fn setup_mark_article_read_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let show_article_action = SimpleAction::new("mark-article-read", glib::VariantTy::new("s").ok());
        show_article_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let update: ReadUpdate = serde_json::from_str(&data).unwrap();

                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        news_flash.set_article_read(&[update.article_id], update.read).unwrap();
                    }
                    if let Some(action) = main_window.lookup_action("update-sidebar") {
                        action.activate(None);
                    }
                }
            }
        });
        show_article_action.set_enabled(true);
        window.add_action(&show_article_action);
    }

    pub fn setup_mark_article_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let show_article_action = SimpleAction::new("mark-article", glib::VariantTy::new("s").ok());
        show_article_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    println!("{}", data);
                    let update: MarkUpdate = serde_json::from_str(&data).unwrap();

                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        news_flash.set_article_marked(&[update.article_id], update.marked).unwrap();
                    }
                    if let Some(action) = main_window.lookup_action("update-sidebar") {
                        action.activate(None);
                    }
                }
            }
        });
        show_article_action.set_enabled(true);
        window.add_action(&show_article_action);
    }
}
