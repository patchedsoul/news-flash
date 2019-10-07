use crate::about_dialog::NewsFlashAbout;
use crate::add_dialog::{AddCategory, AddPopover};
use crate::article_list::{MarkUpdate, ReadUpdate};
use crate::article_view::ArticleView;
use crate::content_page::HeaderSelection;
use crate::content_page::{ContentHeader, ContentPage};
use crate::error_bar::ErrorBar;
use crate::gtk_handle;
use crate::login_screen::{PasswordLogin, WebLogin};
use crate::main_window::{DATA_DIR, MainWindow};
use crate::main_window_state::MainWindowState;
use crate::rename_dialog::RenameDialog;
use crate::responsive::ResponsiveLayout;
use crate::settings::{NewsFlashShortcutWindow, Settings, SettingsDialog};
use crate::sidebar::models::SidebarSelection;
use crate::undo_bar::{UndoActionModel, UndoBar};
use crate::util::{FileUtil, GtkHandle, GtkUtil};
use gio::{ActionMapExt, ApplicationExt, SimpleAction};
use glib::{Variant, VariantTy};
use gtk::{
    self, Application, ApplicationWindow, ButtonExt, DialogExt, FileChooserAction, FileChooserDialog, FileChooserExt,
    FileFilter, GtkWindowExt, GtkWindowExtManual, ResponseType, Stack, StackExt, StackTransitionType,
};
use log::{debug, error, info, warn};
use news_flash::models::{ArticleID, CategoryID, FeedID, LoginData, PluginID};
use news_flash::{NewsFlash, NewsFlashError};
use std::cell::RefCell;
use std::rc::Rc;

pub struct MainWindowActions;

impl MainWindowActions {
    pub fn setup_show_password_page_action(
        window: &ApplicationWindow,
        pw_page: &GtkHandle<PasswordLogin>,
        stack: &Stack,
        header_stack: &Stack,
    ) {
        let stack = stack.clone();
        let header_stack = header_stack.clone();
        let show_pw_page = SimpleAction::new("show-pw-page", VariantTy::new("s").ok());
        let pw_page = pw_page.clone();
        show_pw_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    if let Some(service_meta) = NewsFlash::list_backends().get(&id) {
                        if let Ok(()) = pw_page.borrow_mut().set_service(service_meta.clone()) {
                            header_stack.set_visible_child_name("login");
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
        header_stack: &Stack,
    ) {
        let stack = stack.clone();
        let header_stack = header_stack.clone();
        let oauth_page = oauth_page.clone();
        let show_pw_page = SimpleAction::new("show-oauth-page", VariantTy::new("s").ok());
        show_pw_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    if let Some(service_meta) = NewsFlash::list_backends().get(&id) {
                        if let Ok(()) = oauth_page.borrow_mut().set_service(service_meta.clone()) {
                            header_stack.set_visible_child_name("login");
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
        header_stack: &Stack,
    ) {
        let stack = stack.clone();
        let header_stack = header_stack.clone();
        let show_welcome_page = SimpleAction::new("show-welcome-page", None);
        let pw_page = pw_page.clone();
        let oauth_page = oauth_page.clone();
        show_welcome_page.connect_activate(move |_action, _data| {
            header_stack.set_visible_child_name("welcome");
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
        header_stack: &Stack,
        content_page: &GtkHandle<ContentPage>,
        state: &GtkHandle<MainWindowState>,
        undo_bar: &GtkHandle<UndoBar>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let stack = stack.clone();
        let header_stack = header_stack.clone();
        let content_page = content_page.clone();
        let state = state.clone();
        let undo_bar = undo_bar.clone();
        let error_bar = error_bar.clone();
        let show_content_page = SimpleAction::new("show-content-page", VariantTy::new("s").ok());
        show_content_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    let mut user_name: Option<String> = None;
                    if let Some(api) = &*news_flash.borrow() {
                        user_name = api.user_name();
                    }
                    stack.set_transition_type(StackTransitionType::SlideLeft);
                    stack.set_visible_child_name("content");
                    header_stack.set_visible_child_name("content");

                    if content_page
                        .borrow_mut()
                        .update_sidebar(&news_flash, &state, &undo_bar)
                        .is_err()
                    {
                        error_bar.borrow().simple_message("Failed to update sidebar.");
                    }

                    if content_page.borrow().set_service(&id, user_name).is_err() {
                        error_bar.borrow().simple_message("Failed to set service.");
                    }
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
        let login_action = SimpleAction::new("login", VariantTy::new("s").ok());
        login_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let info: LoginData = serde_json::from_str(&data).expect("Invalid LoginData");
                    let id = match &info {
                        LoginData::OAuth(oauth) => oauth.id.clone(),
                        LoginData::Password(pass) => pass.id.clone(),
                        LoginData::None(id) => id.clone(),
                    };
                    let mut news_flash_lib = match NewsFlash::new(&DATA_DIR, &id) {
                        Ok(news_flash) => news_flash,
                        Err(error) => {
                            match &info {
                                LoginData::OAuth(_) => oauth_page.borrow_mut().show_error(error),
                                LoginData::Password(_) => pw_page.borrow_mut().show_error(error),
                                LoginData::None(_) => {}
                            }
                            return;
                        }
                    };
                    match news_flash_lib.login(info.clone()) {
                        Ok(()) => {
                            // create main obj
                            *news_flash.borrow_mut() = Some(news_flash_lib);

                            // show content page
                            let id = Variant::from(id.to_str());
                            GtkUtil::execute_action_main_window(&main_window, "show-content-page", Some(&id));
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
                                LoginData::None(_) => {
                                    // NOTHING
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

    pub fn setup_sync_action(
        window: &ApplicationWindow,
        application: &Application,
        content_header: &GtkHandle<ContentHeader>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let parent = window.clone();
        let application = application.clone();
        let content_header = content_header.clone();
        let news_flash = news_flash.clone();
        let error_bar = error_bar.clone();
        let sync_action = SimpleAction::new("sync", None);
        sync_action.connect_activate(move |_action, _data| {
            let mut result: Result<i64, NewsFlashError> = Ok(0);
            let mut unread_count = 0;
            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                result = news_flash.sync();
                unread_count = match news_flash.unread_count_all() {
                    Ok(unread_count) => unread_count,
                    Err(_) => 0,
                };
            }
            match result {
                Ok(new_article_count) => {
                    content_header.borrow().finish_sync();
                    GtkUtil::execute_action_main_window(&parent, "update-sidebar", None);
                    GtkUtil::execute_action_main_window(&parent, "update-article-list", None);
                    MainWindow::show_notification(&application, new_article_count, unread_count);
                }
                Err(error) => {
                    content_header.borrow().finish_sync();
                    error_bar.borrow().news_flash_error("Failed to sync.", error);
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
        undo_bar: &GtkHandle<UndoBar>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let undo_bar = undo_bar.clone();
        let error_bar = error_bar.clone();
        let update_sidebar_action = SimpleAction::new("update-sidebar", None);
        update_sidebar_action.connect_activate(move |_action, _data| {
            if content_page
                .borrow_mut()
                .update_sidebar(&news_flash, &state, &undo_bar)
                .is_err()
            {
                error_bar.borrow().simple_message("Failed to update sidebar.");
            }
        });
        update_sidebar_action.set_enabled(true);
        window.add_action(&update_sidebar_action);
    }

    pub fn setup_sidebar_selection_action(
        window: &ApplicationWindow,
        state: &GtkHandle<MainWindowState>,
        responsive_layout: &GtkHandle<ResponsiveLayout>,
    ) {
        let state = state.clone();
        let main_window = window.clone();
        let responsive_layout = responsive_layout.clone();
        let sidebar_selection_action = SimpleAction::new("sidebar-selection", VariantTy::new("s").ok());
        sidebar_selection_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let selection: SidebarSelection = serde_json::from_str(&data).expect("Invalid SidebarSelection");
                    state.borrow_mut().set_sidebar_selection(selection);
                    responsive_layout.borrow().state.borrow_mut().minor_leaflet_selected = true;
                    ResponsiveLayout::process_state_change(&*responsive_layout.borrow());
                    GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
                }
            }
        });
        sidebar_selection_action.set_enabled(true);
        window.add_action(&sidebar_selection_action);
    }

    pub fn setup_headerbar_selection_action(
        window: &ApplicationWindow,
        header: &GtkHandle<ContentHeader>,
        state: &GtkHandle<MainWindowState>,
    ) {
        let state = state.clone();
        let main_window = window.clone();
        let header = header.clone();
        let headerbar_selection_action = SimpleAction::new("headerbar-selection", VariantTy::new("s").ok());
        headerbar_selection_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let new_selection: HeaderSelection = serde_json::from_str(&data).expect("Invalid HeaderSelection");
                    let old_selection = state.borrow().get_header_selection().clone();
                    state.borrow_mut().set_header_selection(new_selection.clone());
                    match new_selection {
                        HeaderSelection::All => header.borrow().select_all_button(),
                        HeaderSelection::Unread => header.borrow().select_unread_button(),
                        HeaderSelection::Marked => header.borrow().select_marked_button(),
                    };
                    GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);

                    let update_sidebar = match old_selection {
                        HeaderSelection::All | HeaderSelection::Unread => match new_selection {
                            HeaderSelection::All | HeaderSelection::Unread => false,
                            HeaderSelection::Marked => true,
                        },
                        HeaderSelection::Marked => match new_selection {
                            HeaderSelection::All | HeaderSelection::Unread => true,
                            HeaderSelection::Marked => false,
                        },
                    };
                    if update_sidebar {
                        GtkUtil::execute_action_main_window(&main_window, "update-sidebar", None);
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
        let search_action = SimpleAction::new("search-term", VariantTy::new("s").ok());
        search_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    if data.is_empty() {
                        state.borrow_mut().set_search_term(None);
                    } else {
                        debug!("Search term: {}", data);
                        state.borrow_mut().set_search_term(Some(data.to_owned()));
                    }

                    GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
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
        undo_bar: &GtkHandle<UndoBar>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let undo_bar = undo_bar.clone();
        let error_bar = error_bar.clone();
        let update_article_list_action = SimpleAction::new("update-article-list", None);
        update_article_list_action.connect_activate(move |_action, _data| {
            if content_page
                .borrow_mut()
                .update_article_list(&news_flash, &state, &undo_bar)
                .is_err()
            {
                error_bar.borrow().simple_message("Failed to update the article list.");
            }
        });
        update_article_list_action.set_enabled(true);
        window.add_action(&update_article_list_action);
    }

    pub fn setup_show_more_articles_action(
        window: &ApplicationWindow,
        state: &GtkHandle<MainWindowState>,
        content_page: &GtkHandle<ContentPage>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        undo_bar: &GtkHandle<UndoBar>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let undo_bar = undo_bar.clone();
        let error_bar = error_bar.clone();
        let show_more_articles_action = SimpleAction::new("show-more-articles", None);
        show_more_articles_action.connect_activate(move |_action, _data| {
            if content_page
                .borrow_mut()
                .load_more_articles(&news_flash, &state, &undo_bar)
                .is_err()
            {
                error_bar.borrow().simple_message("Failed to load more articles.");
            }
        });
        show_more_articles_action.set_enabled(true);
        window.add_action(&show_more_articles_action);
    }

    pub fn setup_show_article_action(
        window: &ApplicationWindow,
        content_page: &GtkHandle<ContentPage>,
        content_header: &GtkHandle<ContentHeader>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        responsive_layout: &GtkHandle<ResponsiveLayout>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let content_page = content_page.clone();
        let content_header = content_header.clone();
        let news_flash = news_flash.clone();
        let responsive_layout = responsive_layout.clone();
        let error_bar = error_bar.clone();
        let show_article_action = SimpleAction::new("show-article", VariantTy::new("s").ok());
        show_article_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let article_id = ArticleID::new(data);
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let article = match news_flash.get_fat_article(&article_id) {
                            Ok(article) => article,
                            Err(error) => {
                                error_bar.borrow().news_flash_error("Failed to read article.", error);
                                return;
                            }
                        };
                        let (feeds, _) = match news_flash.get_feeds() {
                            Ok(res) => res,
                            Err(error) => {
                                error_bar.borrow().news_flash_error("Failed to read feeds.", error);
                                return;
                            }
                        };
                        let feed = match feeds.iter().find(|&f| f.feed_id == article.feed_id) {
                            Some(feed) => feed,
                            None => {
                                error_bar
                                    .borrow()
                                    .simple_message(&format!("Failed to find feed: '{}'", article.feed_id));
                                return;
                            }
                        };
                        content_header.borrow_mut().show_article(Some(&article));
                        content_page.borrow_mut().article_view_show(article, feed);

                        responsive_layout.borrow().state.borrow_mut().major_leaflet_selected = true;
                        ResponsiveLayout::process_state_change(&*responsive_layout.borrow());
                    }
                }
            }
        });
        show_article_action.set_enabled(true);
        window.add_action(&show_article_action);
    }

    pub fn setup_redraw_article_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let redraw_article_action = SimpleAction::new("redraw-article", None);
        redraw_article_action.connect_activate(move |_action, _data| {
            content_page.borrow_mut().article_view_redraw();
        });
        redraw_article_action.set_enabled(true);
        window.add_action(&redraw_article_action);
    }

    pub fn setup_close_article_action(
        window: &ApplicationWindow,
        content_page: &GtkHandle<ContentPage>,
        content_header: &GtkHandle<ContentHeader>,
    ) {
        let content_page = content_page.clone();
        let content_header = content_header.clone();
        let close_article_action = SimpleAction::new("close-article", None);
        close_article_action.connect_activate(move |_action, _data| {
            content_page.borrow_mut().article_view_close();
            content_header.borrow_mut().show_article(None);
        });
        close_article_action.set_enabled(true);
        window.add_action(&close_article_action);
    }

    pub fn setup_toggle_article_read_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let main_window = window.clone();
        let toggle_article_read_action = SimpleAction::new("toggle-article-read", None);
        toggle_article_read_action.connect_activate(move |_action, _data| {
            let visible_article = content_page.borrow().article_view_visible_article();
            if let Some(visible_article) = visible_article {
                let update = ReadUpdate {
                    article_id: visible_article.article_id.clone(),
                    read: visible_article.unread.invert(),
                };
                let update_data = serde_json::to_string(&update).expect("Failed to serialize ReadUpdate");
                let update_data = Variant::from(&update_data);
                GtkUtil::execute_action_main_window(&main_window, "mark-article-read", Some(&update_data));
                GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
            }
        });
        toggle_article_read_action.set_enabled(true);
        window.add_action(&toggle_article_read_action);
    }

    pub fn setup_toggle_article_marked_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let main_window = window.clone();
        let toggle_article_marked_action = SimpleAction::new("toggle-article-marked", None);
        toggle_article_marked_action.connect_activate(move |_action, _data| {
            let visible_article = content_page.borrow().article_view_visible_article();
            if let Some(visible_article) = visible_article {
                let update = MarkUpdate {
                    article_id: visible_article.article_id.clone(),
                    marked: visible_article.marked.invert(),
                };
                let update_data = serde_json::to_string(&update).expect("Failed to serialize MarkUpdate");
                let update_data = Variant::from(&update_data);
                GtkUtil::execute_action_main_window(&main_window, "mark-article", Some(&update_data));
                GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
            }
        });
        toggle_article_marked_action.set_enabled(true);
        window.add_action(&toggle_article_marked_action);
    }

    pub fn setup_mark_article_read_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content_page: &GtkHandle<ContentPage>,
        content_header: &GtkHandle<ContentHeader>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let content_page = content_page.clone();
        let content_header = content_header.clone();
        let error_bar = error_bar.clone();
        let mark_article_read_action = SimpleAction::new("mark-article-read", VariantTy::new("s").ok());
        mark_article_read_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let update: ReadUpdate = serde_json::from_str(&data).expect("Invalid ReadUpdate");

                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        match news_flash.set_article_read(&[update.article_id.clone()], update.read) {
                            Ok(_) => {}
                            Err(error) => {
                                let message = format!("Failed to mark article read: '{}'", update.article_id);
                                error!("{}", message);
                                error_bar.borrow().news_flash_error(&message, error);
                            }
                        }
                    } else {
                        let message = "Failed to borrow NewsFlash.";
                        error!("{}", message);
                        error_bar.borrow().simple_message(message);
                    }

                    GtkUtil::execute_action_main_window(&main_window, "update-sidebar", None);
                    let visible_article = content_page.borrow().article_view_visible_article();
                    if let Some(visible_article) = visible_article {
                        if visible_article.article_id == update.article_id {
                            let mut visible_article = visible_article.clone();
                            visible_article.unread = update.read;
                            content_header.borrow_mut().show_article(Some(&visible_article));
                            content_page
                                .borrow_mut()
                                .article_view_update_visible_article(Some(visible_article.unread), None);
                        }
                    }
                }
            }
        });
        mark_article_read_action.set_enabled(true);
        window.add_action(&mark_article_read_action);
    }

    pub fn setup_mark_article_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content_page: &GtkHandle<ContentPage>,
        content_header: &GtkHandle<ContentHeader>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let content_page = content_page.clone();
        let content_header = content_header.clone();
        let error_bar = error_bar.clone();
        let mark_article_action = SimpleAction::new("mark-article", VariantTy::new("s").ok());
        mark_article_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let update: MarkUpdate = serde_json::from_str(&data).expect("Invalid MarkUpdate");

                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        match news_flash.set_article_marked(&[update.article_id.clone()], update.marked) {
                            Ok(_) => {}
                            Err(error) => {
                                let message = format!("Failed to star article: '{}'", update.article_id);
                                error!("{}", message);
                                error_bar.borrow().news_flash_error(&message, error);
                            }
                        }
                    } else {
                        let message = "Failed to borrow NewsFlash.";
                        error!("{}", message);
                        error_bar.borrow().simple_message(message);
                    }

                    GtkUtil::execute_action_main_window(&main_window, "update-sidebar", None);
                    let visible_article = content_page.borrow().article_view_visible_article();
                    if let Some(visible_article) = visible_article {
                        if visible_article.article_id == update.article_id {
                            let mut visible_article = visible_article.clone();
                            visible_article.marked = update.marked;
                            content_header.borrow_mut().show_article(Some(&visible_article));
                            content_page
                                .borrow_mut()
                                .article_view_update_visible_article(None, Some(visible_article.marked));
                        }
                    }
                }
            }
        });
        mark_article_action.set_enabled(true);
        window.add_action(&mark_article_action);
    }

    pub fn setup_sidebar_set_read_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        state: &GtkHandle<MainWindowState>,
        content_page: &GtkHandle<ContentPage>,
        content_header: &GtkHandle<ContentHeader>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let state = state.clone();
        let content_page = content_page.clone();
        let content_header = content_header.clone();
        let error_bar = error_bar.clone();
        let sidebar_set_read_action = SimpleAction::new("sidebar-set-read", None);
        sidebar_set_read_action.connect_activate(move |_action, _data| {
            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                let sidebar_selection = state.borrow().get_sidebar_selection().clone();

                match sidebar_selection {
                    SidebarSelection::All => match news_flash.set_all_read() {
                        Ok(_) => {}
                        Err(error) => {
                            let message = "Failed to mark all read";
                            error_bar.borrow().news_flash_error(message, error);
                            error!("{}", message);
                        }
                    },
                    SidebarSelection::Cateogry((category_id, _title)) => {
                        match news_flash.set_category_read(&vec![category_id.clone()]) {
                            Ok(_) => {}
                            Err(error) => {
                                let message = format!("Failed to mark category '{}' read", category_id);
                                error_bar.borrow().news_flash_error(&message, error);
                                error!("{}", message);
                            }
                        }
                    }
                    SidebarSelection::Feed((feed_id, _title)) => {
                        match news_flash.set_feed_read(&vec![feed_id.clone()]) {
                            Ok(_) => {}
                            Err(error) => {
                                let message = format!("Failed to mark feed '{}' read", feed_id);
                                error_bar.borrow().news_flash_error(&message, error);
                                error!("{}", message);
                            }
                        }
                    }
                    SidebarSelection::Tag((tag_id, _title)) => match news_flash.set_tag_read(&vec![tag_id.clone()]) {
                        Ok(_) => {}
                        Err(error) => {
                            let message = format!("Failed to mark tag '{}' read", tag_id);
                            error_bar.borrow().news_flash_error(&message, error);
                            error!("{}", message);
                        }
                    },
                }

                let visible_article = content_page.borrow().article_view_visible_article();
                if let Some(visible_article) = visible_article {
                    if let Ok(visible_article) = news_flash.get_fat_article(&visible_article.article_id) {
                        content_header.borrow_mut().show_article(Some(&visible_article));
                        content_page
                            .borrow_mut()
                            .article_view_update_visible_article(Some(visible_article.unread), None);
                    }
                }
            }

            GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
            GtkUtil::execute_action_main_window(&main_window, "update-sidebar", None);
        });
        sidebar_set_read_action.set_enabled(true);
        window.add_action(&sidebar_set_read_action);
    }

    pub fn setup_add_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content: &GtkHandle<ContentPage>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash_handle = news_flash.clone();
        let add_button = content.borrow().sidebar_get_add_button();
        let error_bar = error_bar.clone();
        let add_action = SimpleAction::new("add-feed", None);
        add_action.connect_activate(move |_action, _data| {
            if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
                let news_flash_handle = news_flash_handle.clone();
                let error_message = "Failed to add feed";

                let categories = match news_flash.get_categories() {
                    Ok(categories) => categories,
                    Err(error) => {
                        error!("{}", error_message);
                        error_bar.borrow().news_flash_error(error_message, error);
                        return;
                    }
                };
                let dialog = AddPopover::new(&add_button, categories);
                let error_bar = error_bar.clone();
                dialog.add_button().connect_clicked(move |_button| {
                    let feed_url = match dialog.get_feed_url() {
                        Some(url) => url,
                        None => {
                            error!("{}: No valid url", error_message);
                            error_bar.borrow().simple_message(error_message);
                            return;
                        }
                    };
                    let feed_title = dialog.get_feed_title();
                    let feed_category = dialog.get_category();

                    if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
                        let category_id = match feed_category {
                            AddCategory::New(category_title) => {
                                let category = match news_flash.add_category(&category_title, None, None) {
                                    Ok(category) => category,
                                    Err(error) => {
                                        error!("{}: Can't add Category", error_message);
                                        error_bar.borrow().news_flash_error(error_message, error);
                                        return;
                                    }
                                };
                                Some(category.category_id)
                            }
                            AddCategory::Existing(category_id) => Some(category_id),
                            AddCategory::None => None,
                        };

                        match news_flash.add_feed(&feed_url, feed_title, category_id) {
                            Ok(_) => {}
                            Err(error) => {
                                error!("{}: Can't add Feed", error_message);
                                error_bar.borrow().news_flash_error(error_message, error);
                                return;
                            }
                        }
                    } else {
                        error!("{}: Can't borrow NewsFlash", error_message);
                        error_bar.borrow().simple_message(error_message);
                    }
                });
            }
        });
        add_action.set_enabled(true);
        window.add_action(&add_action);
    }

    pub fn setup_rename_feed_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let error_bar = error_bar.clone();
        let rename_feed_action = SimpleAction::new("rename-feed", VariantTy::new("s").ok());
        rename_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let feed_id = FeedID::new(&data);
                    let dialog_news_flash = news_flash.clone();
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let (feeds, _mappings) = match news_flash.get_feeds() {
                            Ok(result) => result,
                            Err(error) => {
                                error_bar
                                    .borrow()
                                    .news_flash_error("Failed to laod list of feeds.", error);
                                return;
                            }
                        };

                        let feed = match feeds.iter().find(|f| f.feed_id == feed_id).map(|f| f.clone()) {
                            Some(feed) => feed,
                            None => {
                                error_bar
                                    .borrow()
                                    .simple_message(&format!("Failed to find feed '{}'", feed_id));
                                return;
                            }
                        };

                        let dialog =
                            RenameDialog::new(&main_window, &SidebarSelection::Feed((feed_id, feed.label.clone())));
                        let rename_button = dialog.rename_button();
                        let dialog_handle = gtk_handle!(dialog);
                        let main_window = main_window.clone();
                        let error_bar = error_bar.clone();
                        rename_button.connect_clicked(move |_button| {
                            if let Some(news_flash) = dialog_news_flash.borrow_mut().as_mut() {
                                let new_label = match dialog_handle.borrow().new_label() {
                                    Some(label) => label,
                                    None => {
                                        error_bar.borrow().simple_message("No valid title to rename feed.");
                                        dialog_handle.borrow().close();
                                        return;
                                    }
                                };

                                if let Err(error) = news_flash.rename_feed(&feed, &new_label) {
                                    error_bar.borrow().news_flash_error("Failed to rename feed.", error);
                                }

                                dialog_handle.borrow().close();
                            }

                            GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
                            GtkUtil::execute_action_main_window(&main_window, "update-sidebar", None);
                        });
                    }
                }
            }
        });
        rename_feed_action.set_enabled(true);
        window.add_action(&rename_feed_action);
    }

    pub fn setup_rename_category_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let error_bar = error_bar.clone();
        let rename_category_action = SimpleAction::new("rename-category", VariantTy::new("s").ok());
        rename_category_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let category_id = CategoryID::new(&data);
                    let dialog_news_flash = news_flash.clone();
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let categories = match news_flash.get_categories() {
                            Ok(categories) => categories,
                            Err(error) => {
                                error_bar
                                    .borrow()
                                    .news_flash_error("Failed to load list of categories.", error);
                                return;
                            }
                        };

                        let category = match categories
                            .iter()
                            .find(|c| c.category_id == category_id)
                            .map(|c| c.clone())
                        {
                            Some(category) => category,
                            None => {
                                error_bar
                                    .borrow()
                                    .simple_message(&format!("Failed to find category '{}'", category_id));
                                return;
                            }
                        };

                        let dialog = RenameDialog::new(
                            &main_window,
                            &SidebarSelection::Cateogry((category_id, category.label.clone())),
                        );

                        let rename_button = dialog.rename_button();
                        let dialog_handle = gtk_handle!(dialog);
                        let main_window = main_window.clone();
                        let error_bar = error_bar.clone();
                        rename_button.connect_clicked(move |_button| {
                            if let Some(news_flash) = dialog_news_flash.borrow_mut().as_mut() {
                                let new_label = match dialog_handle.borrow().new_label() {
                                    Some(label) => label,
                                    None => {
                                        error_bar.borrow().simple_message("No valid title to rename category.");
                                        return;
                                    }
                                };
                                if let Err(error) = news_flash.rename_category(&category, &new_label) {
                                    error_bar.borrow().news_flash_error("Failed to rename category.", error);
                                }
                                dialog_handle.borrow().close();
                            }

                            GtkUtil::execute_action_main_window(&main_window, "update-sidebar", None);
                        });
                    }
                }
            }
        });
        rename_category_action.set_enabled(true);
        window.add_action(&rename_category_action);
    }

    pub fn setup_delete_selection_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let main_window = window.clone();
        let delete_selection_action = SimpleAction::new("delete-selection", None);
        delete_selection_action.connect_activate(move |_action, _data| {
            let selection = content_page.borrow().sidebar_get_selection();
            let undo_action = match selection {
                SidebarSelection::All => {
                    warn!("Trying to delete item while 'All Articles' is selected");
                    None
                }
                SidebarSelection::Feed((feed_id, label)) => Some(UndoActionModel::DeleteFeed((feed_id, label))),
                SidebarSelection::Cateogry((category_id, label)) => {
                    Some(UndoActionModel::DeleteCategory((category_id, label)))
                }
                SidebarSelection::Tag((tag_id, label)) => Some(UndoActionModel::DeleteTag((tag_id, label))),
            };
            if let Some(undo_action) = undo_action {
                let json = serde_json::to_string(&undo_action).expect("Failed to serialize UndoActionModel");
                GtkUtil::execute_action_main_window(
                    &main_window,
                    "enqueue-undoable-action",
                    Some(&Variant::from(&json)),
                );
            }
        });
        delete_selection_action.set_enabled(true);
        window.add_action(&delete_selection_action);
    }

    pub fn setup_enqueue_undoable_action(
        window: &ApplicationWindow,
        undo_bar: &GtkHandle<UndoBar>,
        content_page: &GtkHandle<ContentPage>,
        state: &GtkHandle<MainWindowState>,
    ) {
        let undo_bar = undo_bar.clone();
        let state = state.clone();
        let content_page = content_page.clone();
        let enqueue_undoable_action = SimpleAction::new("enqueue-undoable-action", VariantTy::new("s").ok());
        enqueue_undoable_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let action: UndoActionModel =
                        serde_json::from_str(&data).expect("Failed to deserialize UndoActionModel.");

                    let select_all_button = match content_page.borrow().sidebar_get_selection() {
                        SidebarSelection::All => false,
                        SidebarSelection::Cateogry((selected_id, _label)) => match &action {
                            UndoActionModel::DeleteCategory((delete_id, _label)) => {
                                if &selected_id == delete_id {
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        },
                        SidebarSelection::Feed((selected_id, _label)) => match &action {
                            UndoActionModel::DeleteFeed((delete_id, _label)) => {
                                if &selected_id == delete_id {
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        },
                        SidebarSelection::Tag((selected_id, _label)) => match &action {
                            UndoActionModel::DeleteTag((delete_id, _label)) => {
                                if &selected_id == delete_id {
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        },
                    };
                    if select_all_button {
                        state.borrow_mut().set_sidebar_selection(SidebarSelection::All);
                        content_page.borrow().sidebar_select_all_button_no_update();
                    }

                    info!("enque new undoable action: {}", action);
                    undo_bar.borrow().add_action(action);
                }
            }
        });
        enqueue_undoable_action.set_enabled(true);
        window.add_action(&enqueue_undoable_action);
    }

    pub fn setup_delete_feed_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let error_bar = error_bar.clone();
        let delete_feed_action = SimpleAction::new("delete-feed", VariantTy::new("s").ok());
        delete_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let feed_id = FeedID::new(&data);
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let (feeds, _mappings) = match news_flash.get_feeds() {
                            Ok(res) => res,
                            Err(error) => {
                                error_bar.borrow().news_flash_error("Failed to delete feed.", error);
                                return;
                            }
                        };

                        if let Some(feed) = feeds.iter().find(|f| f.feed_id == feed_id).map(|f| f.clone()) {
                            info!("delete feed '{}' (id: {})", feed.label, feed.feed_id);
                            if let Err(error) = news_flash.remove_feed(&feed) {
                                error_bar.borrow().news_flash_error("Failed to delete feed.", error);
                            }
                        } else {
                            error_bar.borrow().simple_message(&format!(
                                "Failed to delete feed: feed with id '{}' not found.",
                                feed_id
                            ));
                            error!("feed not found: {}", feed_id);
                        }
                    }
                }
            }
        });
        delete_feed_action.set_enabled(true);
        window.add_action(&delete_feed_action);
    }

    pub fn setup_delete_category_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let news_flash = news_flash.clone();
        let error_bar = error_bar.clone();
        let delete_feed_action = SimpleAction::new("delete-category", VariantTy::new("s").ok());
        delete_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let category_id = CategoryID::new(&data);
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let categories = match news_flash.get_categories() {
                            Ok(res) => res,
                            Err(error) => {
                                error_bar.borrow().news_flash_error("Failed to delete category.", error);
                                return;
                            }
                        };

                        if let Some(category) = categories
                            .iter()
                            .find(|c| c.category_id == category_id)
                            .map(|c| c.clone())
                        {
                            info!("delete category '{}' (id: {})", category.label, category.category_id);
                            if let Err(error) = news_flash.remove_category(&category, true) {
                                error_bar.borrow().news_flash_error("Failed to delete category.", error);
                            }
                        } else {
                            error_bar.borrow().simple_message(&format!(
                                "Failed to delete category: category with id '{}' not found.",
                                category_id
                            ));
                            error!("category not found: {}", category_id);
                        }
                    }
                }
            }
        });
        delete_feed_action.set_enabled(true);
        window.add_action(&delete_feed_action);
    }

    pub fn setup_select_next_article_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let next_article_action = SimpleAction::new("next-article", None);
        next_article_action.connect_activate(move |_action, _data| {
            content_page.borrow().select_next_article();
        });
        next_article_action.set_enabled(true);
        window.add_action(&next_article_action);
    }

    pub fn setup_select_prev_article_action(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>) {
        let content_page = content_page.clone();
        let prev_article_action = SimpleAction::new("prev-article", None);
        prev_article_action.connect_activate(move |_action, _data| {
            content_page.borrow().select_prev_article();
        });
        prev_article_action.set_enabled(true);
        window.add_action(&prev_article_action);
    }

    pub fn setup_about_action(window: &ApplicationWindow) {
        let main_window = window.clone();
        let about_action = SimpleAction::new("about", None);
        about_action.connect_activate(move |_action, _data| {
            let dialog = NewsFlashAbout::new(Some(&main_window)).widget();
            dialog.present();
        });
        about_action.set_enabled(true);
        window.add_action(&about_action);
    }

    pub fn setup_settings_action(
        window: &ApplicationWindow,
        settings: &GtkHandle<Settings>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let main_window = window.clone();
        let settings = settings.clone();
        let error_bar = error_bar.clone();
        let settings_action = SimpleAction::new("settings", None);
        settings_action.connect_activate(move |_action, _data| {
            let dialog = SettingsDialog::new(&main_window, &settings, &error_bar).widget();
            dialog.present();
        });
        settings_action.set_enabled(true);
        window.add_action(&settings_action);
    }

    pub fn setup_shortcut_window_action(window: &ApplicationWindow, settings: &GtkHandle<Settings>) {
        let main_window = window.clone();
        let settings = settings.clone();
        let settings_action = SimpleAction::new("shortcuts", None);
        settings_action.connect_activate(move |_action, _data| {
            let dialog = NewsFlashShortcutWindow::new(&main_window, &settings).widget();
            dialog.present();
        });
        settings_action.set_enabled(true);
        window.add_action(&settings_action);
    }

    pub fn setup_quit_action(window: &ApplicationWindow, app: &Application) {
        let main_window = window.clone();
        let app = app.clone();
        let quit_action = SimpleAction::new("quit", None);
        quit_action.connect_activate(move |_action, _data| {
            // FIXME: check for ongoing sync
            main_window.close();
            app.quit();
        });
        quit_action.set_enabled(true);
        window.add_action(&quit_action);
    }

    pub fn setup_export_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let main_window = window.clone();
        let news_flash = news_flash.clone();
        let error_bar = error_bar.clone();
        let export_action = SimpleAction::new("export", None);
        export_action.connect_activate(move |_action, _data| {
            let dialog = FileChooserDialog::with_buttons(
                Some("Export OPML"),
                Some(&main_window),
                FileChooserAction::Save,
                &vec![("Cancel", ResponseType::Cancel), ("Save", ResponseType::Ok)],
            );

            let filter = FileFilter::new();
            filter.add_pattern("*.OPML");
            filter.add_pattern("*.opml");
            filter.add_mime_type("application/xml");
            filter.add_mime_type("text/xml");
            filter.add_mime_type("text/x-opml");
            filter.set_name(Some("OPML"));
            dialog.add_filter(&filter);
            dialog.set_filter(&filter);
            dialog.set_current_name("NewsFlash.OPML");

            match ResponseType::from(dialog.run()) {
                ResponseType::Ok => {
                    if let Some(news_flash) = news_flash.borrow().as_ref() {
                        let opml = match news_flash.export_opml() {
                            Ok(opml) => opml,
                            Err(error) => {
                                error_bar.borrow().news_flash_error("Failed to get OPML data.", error);
                                return;
                            }
                        };
                        if let Some(filename) = dialog.get_filename() {
                            if FileUtil::write_text_file(&filename, &opml).is_err() {
                                error_bar.borrow().simple_message("Failed to write OPML data to disc.")
                            }
                        }
                    }
                }
                _ => {}
            }

            dialog.emit_close();
        });
        export_action.set_enabled(true);
        window.add_action(&export_action);
    }

    pub fn setup_export_article_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content_page: &GtkHandle<ContentPage>,
        settings: &GtkHandle<Settings>,
        error_bar: &GtkHandle<ErrorBar>,
    ) {
        let main_window = window.clone();
        let news_flash = news_flash.clone();
        let content_page = content_page.clone();
        let settings = settings.clone();
        let error_bar = error_bar.clone();
        let export_article_action = SimpleAction::new("export-article", None);
        export_article_action.connect_activate(move |_action, _data| {
            if let Some(article) = content_page.borrow().article_view_visible_article() {
                let dialog = FileChooserDialog::with_buttons(
                    Some("Export Article"),
                    Some(&main_window),
                    FileChooserAction::Save,
                    &vec![("Cancel", ResponseType::Cancel), ("Save", ResponseType::Ok)],
                );

                let filter = FileFilter::new();
                filter.add_pattern("*.html");
                filter.add_mime_type("text/html");
                filter.set_name(Some("HTML"));
                dialog.add_filter(&filter);
                dialog.set_filter(&filter);
                if let Some(title) = &article.title {
                    dialog.set_current_name(&format!("{}.html", title));
                } else {
                    dialog.set_current_name("Article.html");
                }

                match ResponseType::from(dialog.run()) {
                    ResponseType::Ok => {
                        if let Some(news_flash) = news_flash.borrow().as_ref() {
                            let article = match news_flash.article_download_images(&article.article_id) {
                                Ok(opml) => opml,
                                Err(error) => {
                                    error_bar
                                        .borrow()
                                        .news_flash_error("Failed to downlaod article images.", error);
                                    return;
                                }
                            };

                            let (feeds, _) = match news_flash.get_feeds() {
                                Ok(opml) => opml,
                                Err(error) => {
                                    error_bar
                                        .borrow()
                                        .news_flash_error("Failed to load feeds from db.", error);
                                    return;
                                }
                            };
                            let feed = match feeds.iter().find(|&f| f.feed_id == article.feed_id) {
                                Some(feed) => feed,
                                None => {
                                    error_bar.borrow().simple_message("Failed to find specific feed.");
                                    return;
                                }
                            };
                            if let Some(filename) = dialog.get_filename() {
                                let html = ArticleView::build_article_static(
                                    "article",
                                    &article,
                                    &feed.label,
                                    &settings,
                                    None,
                                    None,
                                );
                                if FileUtil::write_text_file(&filename, &html).is_err() {
                                    error_bar.borrow().simple_message("Failed to write OPML data to disc.")
                                }
                            }
                        }
                    }
                    _ => {}
                }

                dialog.emit_close();
            }
        });
        export_article_action.set_enabled(true);
        window.add_action(&export_article_action);
    }
}
