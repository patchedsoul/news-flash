use crate::about_dialog::NewsFlashAbout;
use crate::add_dialog::AddPopover;
use crate::article_list::{MarkUpdate, ReadUpdate};
use crate::content_page::HeaderSelection;
use crate::content_page::{ContentHeader, ContentPage};
use crate::error_dialog::ErrorDialog;
use crate::gtk_handle;
use crate::login_screen::{PasswordLogin, WebLogin};
use crate::main_window::DATA_DIR;
use crate::main_window_state::MainWindowState;
use crate::rename_dialog::RenameDialog;
use crate::responsive::ResponsiveLayout;
use crate::settings::{NewsFlashShortcutWindow, Settings, SettingsDialog};
use crate::sidebar::models::SidebarSelection;
use crate::undo_bar::{UndoActionModel, UndoBar};
use crate::util::{FileUtil, GtkHandle};
use gio::{ActionExt, ActionMapExt, ApplicationExt, SimpleAction};
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
    ) {
        let news_flash = news_flash.clone();
        let stack = stack.clone();
        let header_stack = header_stack.clone();
        let content_page = content_page.clone();
        let state = state.clone();
        let undo_bar = undo_bar.clone();
        let show_content_page = SimpleAction::new("show-content-page", VariantTy::new("s").ok());
        show_content_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    let mut user_name: Option<String> = None;
                    if let Some(api) = &*news_flash.borrow() {
                        user_name = api.user_name();
                    }
                    content_page.borrow_mut().update_sidebar(&news_flash, &state, &undo_bar);
                    content_page.borrow().set_service(&id, user_name).unwrap();
                    header_stack.set_visible_child_name("content");
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
        let login_action = SimpleAction::new("login", VariantTy::new("s").ok());
        login_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let info: LoginData = serde_json::from_str(&data).unwrap();
                    let id = match &info {
                        LoginData::OAuth(oauth) => oauth.id.clone(),
                        LoginData::Password(pass) => pass.id.clone(),
                        LoginData::None(id) => id.clone(),
                    };
                    let mut news_flash_lib = NewsFlash::new(&DATA_DIR, &id).unwrap();
                    match news_flash_lib.login(info.clone()) {
                        Ok(()) => {
                            // create main obj
                            *news_flash.borrow_mut() = Some(news_flash_lib);

                            // show content page
                            if let Some(action) = main_window.lookup_action("show-content-page") {
                                let id = Variant::from(id.to_str());
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
        content_header: &GtkHandle<ContentHeader>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
    ) {
        let parent = window.clone();
        let content_header = content_header.clone();
        let news_flash = news_flash.clone();
        let sync_action = SimpleAction::new("sync", None);
        sync_action.connect_activate(move |_action, _data| {
            let mut result: Result<(), NewsFlashError> = Ok(());
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
                    content_header.borrow().finish_sync();
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
        undo_bar: &GtkHandle<UndoBar>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let undo_bar = undo_bar.clone();
        let sync_action = SimpleAction::new("update-sidebar", None);
        sync_action.connect_activate(move |_action, _data| {
            content_page.borrow_mut().update_sidebar(&news_flash, &state, &undo_bar);
        });
        sync_action.set_enabled(true);
        window.add_action(&sync_action);
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
                    let selection: SidebarSelection = serde_json::from_str(&data).unwrap();
                    state.borrow_mut().set_sidebar_selection(selection);
                    responsive_layout.borrow().state.borrow_mut().minor_leaflet_selected = true;
                    ResponsiveLayout::process_state_change(&*responsive_layout.borrow());
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
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
                    let new_selection: HeaderSelection = serde_json::from_str(&data).unwrap();
                    let old_selection = state.borrow().get_header_selection().clone();
                    state.borrow_mut().set_header_selection(new_selection.clone());
                    match new_selection {
                        HeaderSelection::All => header.borrow().select_all_button(),
                        HeaderSelection::Unread => header.borrow().select_unread_button(),
                        HeaderSelection::Marked => header.borrow().select_marked_button(),
                    };
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
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
        undo_bar: &GtkHandle<UndoBar>,
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let undo_bar = undo_bar.clone();
        let update_article_list_action = SimpleAction::new("update-article-list", None);
        update_article_list_action.connect_activate(move |_action, _data| {
            content_page
                .borrow_mut()
                .update_article_list(&news_flash, &state, &undo_bar);
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
    ) {
        let state = state.clone();
        let content_page = content_page.clone();
        let news_flash = news_flash.clone();
        let undo_bar = undo_bar.clone();
        let show_more_articles_action = SimpleAction::new("show-more-articles", None);
        show_more_articles_action.connect_activate(move |_action, _data| {
            content_page
                .borrow_mut()
                .load_more_articles(&news_flash, &state, &undo_bar)
                .unwrap();
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
    ) {
        let content_page = content_page.clone();
        let content_header = content_header.clone();
        let news_flash = news_flash.clone();
        let responsive_layout = responsive_layout.clone();
        let show_article_action = SimpleAction::new("show-article", VariantTy::new("s").ok());
        show_article_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let article_id = ArticleID::new(data);
                    content_page
                        .borrow_mut()
                        .article_view_show(&article_id, &news_flash)
                        .unwrap();
                    content_header.borrow().set_article_header_sensitive(true);
                    responsive_layout.borrow().state.borrow_mut().major_leaflet_selected = true;
                    ResponsiveLayout::process_state_change(&*responsive_layout.borrow());
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
            content_page.borrow_mut().article_view_redraw().unwrap();
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
            content_page.borrow_mut().article_view_close().unwrap();
            content_header.borrow().set_article_header_sensitive(false);
        });
        close_article_action.set_enabled(true);
        window.add_action(&close_article_action);
    }

    pub fn setup_mark_article_read_action(window: &ApplicationWindow, news_flash: &GtkHandle<Option<NewsFlash>>) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let mark_article_read_action = SimpleAction::new("mark-article-read", VariantTy::new("s").ok());
        mark_article_read_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let update: ReadUpdate = serde_json::from_str(&data).unwrap();

                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        news_flash
                            .set_article_read(&[update.article_id.clone()], update.read)
                            .unwrap();
                    }
                    if let Some(action) = main_window.lookup_action("update-sidebar") {
                        action.activate(None);
                    }
                }
            }
        });
        mark_article_read_action.set_enabled(true);
        window.add_action(&mark_article_read_action);
    }

    pub fn setup_mark_article_action(window: &ApplicationWindow, news_flash: &GtkHandle<Option<NewsFlash>>) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let mark_article_action = SimpleAction::new("mark-article", VariantTy::new("s").ok());
        mark_article_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let update: MarkUpdate = serde_json::from_str(&data).unwrap();

                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        news_flash
                            .set_article_marked(&[update.article_id.clone()], update.marked)
                            .unwrap();
                    }
                    if let Some(action) = main_window.lookup_action("update-sidebar") {
                        action.activate(None);
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
    ) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let state = state.clone();
        let sidebar_set_read_action = SimpleAction::new("sidebar-set-read", None);
        sidebar_set_read_action.connect_activate(move |_action, _data| {
            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                let sidebar_selection = state.borrow().get_sidebar_selection().clone();

                match sidebar_selection {
                    SidebarSelection::All => {
                        news_flash.set_all_read().unwrap();
                    }
                    SidebarSelection::Cateogry((category_id, _title)) => {
                        news_flash.set_category_read(&vec![category_id]).unwrap();
                    }
                    SidebarSelection::Feed((feed_id, _title)) => news_flash.set_feed_read(&vec![feed_id]).unwrap(),
                    SidebarSelection::Tag((tag_id, _title)) => {
                        news_flash.set_tag_read(&vec![tag_id]).unwrap();
                    }
                }
            }

            if let Some(action) = main_window.lookup_action("update-article-list") {
                action.activate(None);
            }
            if let Some(action) = main_window.lookup_action("update-sidebar") {
                action.activate(None);
            }
        });
        sidebar_set_read_action.set_enabled(true);
        window.add_action(&sidebar_set_read_action);
    }

    pub fn setup_add_action(
        window: &ApplicationWindow,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content: &GtkHandle<ContentPage>,
    ) {
        let news_flash = news_flash.clone();
        let add_button = content.borrow().sidebar_get_add_button();
        let add_action = SimpleAction::new("add-feed", None);
        add_action.connect_activate(move |_action, _data| {
            if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                let categories = news_flash.get_categories().unwrap();
                let _dialog = AddPopover::new(&add_button, &categories);
            }
        });
        add_action.set_enabled(true);
        window.add_action(&add_action);
    }

    pub fn setup_rename_feed_action(window: &ApplicationWindow, news_flash: &GtkHandle<Option<NewsFlash>>) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let rename_feed_action = SimpleAction::new("rename-feed", VariantTy::new("s").ok());
        rename_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let feed_id = FeedID::new(&data);
                    let dialog_news_flash = news_flash.clone();
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let (feeds, _mappings) = news_flash.get_feeds().unwrap();
                        let feed = feeds.iter().find(|f| f.feed_id == feed_id).map(|f| f.clone()).unwrap();
                        let dialog =
                            RenameDialog::new(&main_window, &SidebarSelection::Feed((feed_id, feed.label.clone())));
                        let rename_button = dialog.rename_button();
                        let dialog_handle = gtk_handle!(dialog);
                        let main_window = main_window.clone();
                        rename_button.connect_clicked(move |_button| {
                            if let Some(news_flash) = dialog_news_flash.borrow_mut().as_mut() {
                                let new_label = dialog_handle.borrow().new_label().unwrap();
                                news_flash.rename_feed(&feed, &new_label).unwrap();
                                dialog_handle.borrow().close();
                            }
                            if let Some(action) = main_window.lookup_action("update-sidebar") {
                                action.activate(None);
                            }
                            if let Some(action) = main_window.lookup_action("update-article-list") {
                                action.activate(None);
                            }
                        });
                    }
                }
            }
        });
        rename_feed_action.set_enabled(true);
        window.add_action(&rename_feed_action);
    }

    pub fn setup_rename_category_action(window: &ApplicationWindow, news_flash: &GtkHandle<Option<NewsFlash>>) {
        let news_flash = news_flash.clone();
        let main_window = window.clone();
        let rename_category_action = SimpleAction::new("rename-category", VariantTy::new("s").ok());
        rename_category_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let category_id = CategoryID::new(&data);
                    let dialog_news_flash = news_flash.clone();
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let categories = news_flash.get_categories().unwrap();
                        let category = categories
                            .iter()
                            .find(|c| c.category_id == category_id)
                            .map(|c| c.clone())
                            .unwrap();
                        let dialog = RenameDialog::new(
                            &main_window,
                            &SidebarSelection::Cateogry((category_id, category.label.clone())),
                        );
                        let rename_button = dialog.rename_button();
                        let dialog_handle = gtk_handle!(dialog);
                        let main_window = main_window.clone();
                        rename_button.connect_clicked(move |_button| {
                            if let Some(news_flash) = dialog_news_flash.borrow_mut().as_mut() {
                                let new_label = dialog_handle.borrow().new_label().unwrap();
                                news_flash.rename_category(&category, &new_label).unwrap();
                                dialog_handle.borrow().close();
                            }
                            if let Some(action) = main_window.lookup_action("update-sidebar") {
                                action.activate(None);
                            }
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
        let delete_selection_action = SimpleAction::new("delete-selection-action", None);
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
                if let Some(action) = main_window.lookup_action("enqueue-undoable-action") {
                    if let Ok(json) = serde_json::to_string(&undo_action) {
                        let json = Variant::from(&json);
                        action.activate(Some(&json));
                    }
                }
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
                    let action: UndoActionModel = serde_json::from_str(&data).unwrap();

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

    pub fn setup_delete_feed_action(window: &ApplicationWindow, news_flash: &GtkHandle<Option<NewsFlash>>) {
        let news_flash = news_flash.clone();
        let delete_feed_action = SimpleAction::new("delete-feed", VariantTy::new("s").ok());
        delete_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let feed_id = FeedID::new(&data);
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let (feeds, _mappings) = news_flash.get_feeds().unwrap();

                        if let Some(feed) = feeds.iter().find(|f| f.feed_id == feed_id).map(|f| f.clone()) {
                            info!("delete feed '{}' (id: {})", feed.label, feed.feed_id);
                        //news_flash.remove_feed(&feed).unwrap();
                        } else {
                            // FIXME: error handling
                            error!("feed not found: {}", feed_id);
                        }
                    }
                }
            }
        });
        delete_feed_action.set_enabled(true);
        window.add_action(&delete_feed_action);
    }

    pub fn setup_delete_category_action(window: &ApplicationWindow, news_flash: &GtkHandle<Option<NewsFlash>>) {
        let news_flash = news_flash.clone();
        let delete_feed_action = SimpleAction::new("delete-category", VariantTy::new("s").ok());
        delete_feed_action.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(data) = data.get_str() {
                    let category_id = CategoryID::new(&data);
                    if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                        let categories = news_flash.get_categories().unwrap();

                        if let Some(category) = categories
                            .iter()
                            .find(|c| c.category_id == category_id)
                            .map(|c| c.clone())
                        {
                            info!("delete category '{}' (id: {})", category.label, category.category_id);
                        //news_flash.remove_feed(&feed).unwrap();
                        } else {
                            // FIXME: error handling
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

    pub fn setup_settings_action(window: &ApplicationWindow, settings: &GtkHandle<Settings>) {
        let main_window = window.clone();
        let settings = settings.clone();
        let settings_action = SimpleAction::new("settings", None);
        settings_action.connect_activate(move |_action, _data| {
            let dialog = SettingsDialog::new(&main_window, &settings).widget();
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

    pub fn setup_export_action(window: &ApplicationWindow, news_flash: &GtkHandle<Option<NewsFlash>>) {
        let main_window = window.clone();
        let news_flash = news_flash.clone();
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
                        let opml = news_flash.export_opml().unwrap();
                        if let Some(filename) = dialog.get_filename() {
                            FileUtil::write_text_file(&filename, &opml).unwrap();
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
}
