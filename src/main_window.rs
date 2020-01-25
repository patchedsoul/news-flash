use crate::about_dialog::APP_NAME;
use crate::app::Action;
use crate::article_list::{MarkUpdate, ReadUpdate};
use crate::config::{APP_ID, PROFILE};
use crate::content_page::{ContentHeader, ContentPage, HeaderSelection};
use crate::error_bar::ErrorBar;
use crate::gtk_handle;
use crate::login_screen::{LoginHeaderbar, PasswordLogin, WebLogin};
use crate::main_window_state::MainWindowState;
use crate::responsive::ResponsiveLayout;
use crate::settings::{Keybindings, Settings};
use crate::sidebar::models::SidebarSelection;
use crate::undo_bar::{UndoActionModel, UndoBar};
use crate::util::{BuilderHelper, GtkHandle, GtkUtil, Util, GTK_CSS_ERROR, GTK_RESOURCE_FILE_ERROR, RUNTIME_ERROR};
use crate::welcome_screen::{WelcomeHeaderbar, WelcomePage};
use crate::Resources;
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use gdk::EventKey;
use glib::{self, futures::FutureExt, Sender};
use gtk::{
    self, ApplicationWindow, CssProvider, CssProviderExt, GtkWindowExt, Inhibit, Settings as GtkSettings, SettingsExt,
    Stack, StackExt, StackTransitionType, StyleContext, StyleContextExt, WidgetExt,
};
use log::{error, warn};
use news_flash::models::{ArticleID, PluginID};
use news_flash::{NewsFlash, NewsFlashError};
use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::runtime::Runtime;

const CONTENT_PAGE: &str = "content";

pub struct MainWindow {
    pub widget: ApplicationWindow,
    error_bar: ErrorBar,
    undo_bar: UndoBar,
    pub oauth_login_page: Rc<WebLogin>,
    pub password_login_page: Rc<PasswordLogin>,
    pub content_page: Rc<RwLock<ContentPage>>,
    pub content_header: Rc<ContentHeader>,
    stack: Stack,
    header_stack: Stack,
    responsive_layout: ResponsiveLayout,
    state: Arc<RwLock<MainWindowState>>,
    sender: Sender<Action>,
}

impl MainWindow {
    pub fn new(
        settings: &Arc<RwLock<Settings>>,
        sender: Sender<Action>,
        shutdown_in_progress: Arc<RwLock<bool>>,
    ) -> Self {
        GtkUtil::register_symbolic_icons();
        let provider_handle = gtk_handle!(CssProvider::new());

        if let Some(gtk_settings) = GtkSettings::get_default() {
            gtk_settings.set_property_gtk_application_prefer_dark_theme(settings.read().get_prefer_dark_theme());
        }

        // setup CSS for window
        Self::load_css(&provider_handle);

        let builder = BuilderHelper::new("main_window");
        let window = builder.get::<ApplicationWindow>("main_window");
        let stack = builder.get::<Stack>("main_stack");
        let header_stack = builder.get::<Stack>("header_stack");
        let undo_bar = UndoBar::new(&builder, sender.clone());
        let error_bar = ErrorBar::new(&builder, sender.clone());

        let responsive_layout = ResponsiveLayout::new(&builder);

        let _login_header = LoginHeaderbar::new(&builder, sender.clone());
        let _welcome_header = WelcomeHeaderbar::new(&builder);
        let content_header = Rc::new(ContentHeader::new(&builder, sender.clone()));

        window.set_icon_name(Some(APP_ID));
        window.set_title(APP_NAME);
        if PROFILE == "Devel" {
            window.get_style_context().add_class("devel");
        }

        let delete_event_settings = settings.clone();
        let sender_clone = sender.clone();
        let main_stack = stack.clone();
        window.connect_delete_event(move |win, _| {
            if *shutdown_in_progress.read() {
                win.hide_on_delete();
                return Inhibit(true);
            }
            if delete_event_settings.read().get_keep_running_in_background() {
                if let Some(visible_child) = main_stack.get_visible_child_name() {
                    if visible_child == CONTENT_PAGE {
                        win.hide_on_delete();
                    } else {
                        Util::send(&sender_clone, Action::QueueQuit);
                    }
                }
            } else {
                Util::send(&sender_clone, Action::QueueQuit);
            }

            Inhibit(true)
        });

        // setup pages
        let _welcome = WelcomePage::new(&builder, sender.clone());
        let password_login_page = Rc::new(PasswordLogin::new(&builder, sender.clone()));
        let oauth_login_page = Rc::new(WebLogin::new(&builder, sender.clone()));
        let content_page = Rc::new(RwLock::new(ContentPage::new(&builder, &settings, sender.clone())));
        let state = Arc::new(RwLock::new(MainWindowState::new()));

        Self::setup_shortcuts(&window, &sender, &content_page, &stack, &settings, &content_header);

        if let Some(gtk_settings) = GtkSettings::get_default() {
            gtk_settings.set_property_gtk_application_prefer_dark_theme(settings.read().get_prefer_dark_theme());

            let provider = provider_handle.clone();
            let sender_clone = sender.clone();
            gtk_settings.connect_property_gtk_application_prefer_dark_theme_notify(move |_settings| {
                Self::load_css(&provider);
                Util::send(&sender_clone, Action::RedrawArticle);
            });
        }

        MainWindow {
            widget: window,
            error_bar,
            undo_bar: undo_bar,
            content_page,
            oauth_login_page,
            password_login_page,
            content_header,
            stack,
            header_stack,
            responsive_layout,
            state,
            sender,
        }
    }

    fn setup_shortcuts(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        content_page: &Rc<RwLock<ContentPage>>,
        main_stack: &Stack,
        settings: &Arc<RwLock<Settings>>,
        content_header: &Rc<ContentHeader>,
    ) {
        let main_stack = main_stack.clone();
        let sender = sender.clone();
        let settings = settings.clone();
        let content_page = content_page.clone();
        let main_window = window.clone();
        let content_header = content_header.clone();
        window.connect_key_press_event(move |_widget, event| {
            // ignore shortcuts when not on content page
            if let Some(visible_child) = main_stack.get_visible_child_name() {
                if visible_child != CONTENT_PAGE {
                    return Inhibit(false);
                }
            }

            // ignore shortcuts when typing in search entry
            if content_header.is_search_focused() {
                return Inhibit(false);
            }

            if Self::check_shortcut("shortcuts", &settings, event) {
                Util::send(&sender, Action::ShowShortcutWindow);
            }

            if Self::check_shortcut("refresh", &settings, event) {
                Util::send(&sender, Action::Sync);
            }

            if Self::check_shortcut("quit", &settings, event) {
                Util::send(&sender, Action::QueueQuit);
            }

            if Self::check_shortcut("search", &settings, event) {
                content_header.focus_search();
            }

            if Self::check_shortcut("all_articles", &settings, event) {
                content_header.select_all_button();
            }

            if Self::check_shortcut("only_unread", &settings, event) {
                content_header.select_unread_button();
            }

            if Self::check_shortcut("only_starred", &settings, event) {
                content_header.select_marked_button();
            }

            if Self::check_shortcut("next_article", &settings, event) {
                Util::send(&sender, Action::SidebarSelectNext);
            }

            if Self::check_shortcut("previous_article", &settings, event) {
                Util::send(&sender, Action::SidebarSelectPrev);
            }

            if Self::check_shortcut("toggle_category_expanded", &settings, event) {
                content_page.read().sidebar_expand_collase_category();
            }

            if Self::check_shortcut("toggle_read", &settings, event) {
                let article_model = content_page.read().get_selected_article_model();
                if let Some(article_model) = article_model {
                    let update = ReadUpdate {
                        article_id: article_model.id.clone(),
                        read: article_model.read.invert(),
                    };

                    Util::send(&sender, Action::MarkArticleRead(update));
                    Util::send(&sender, Action::UpdateArticleList);
                }
            }

            if Self::check_shortcut("toggle_marked", &settings, event) {
                let article_model = content_page.read().get_selected_article_model();
                if let Some(article_model) = article_model {
                    let update = MarkUpdate {
                        article_id: article_model.id.clone(),
                        marked: article_model.marked.invert(),
                    };

                    Util::send(&sender, Action::MarkArticle(update));
                    Util::send(&sender, Action::UpdateArticleList);
                }
            }

            if Self::check_shortcut("open_browser", &settings, event) {
                let article_model = content_page.read().get_selected_article_model();
                if let Some(article_model) = article_model {
                    if let Some(url) = article_model.url {
                        if gtk::show_uri_on_window(Some(&main_window), url.get().as_str(), 0).is_err() {
                            Util::send(
                                &sender,
                                Action::ErrorSimpleMessage("Failed to open URL in browser.".to_owned()),
                            );
                        }
                    } else {
                        warn!("Open selected article in browser: No url available.")
                    }
                } else {
                    warn!("Open selected article in browser: No article Selected.")
                }
            }

            if Self::check_shortcut("next_item", &settings, event)
                && content_page.read().sidebar_select_next_item().is_err()
            {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select next item in sidebar.".to_owned()),
                );
            }

            if Self::check_shortcut("previous_item", &settings, event)
                && content_page.read().sidebar_select_prev_item().is_err()
            {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select previous item in sidebar.".to_owned()),
                );
            }

            if Self::check_shortcut("scroll_up", &settings, event)
                && content_page.read().article_view_scroll_diff(-150.0).is_err()
            {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select scroll article view up.".to_owned()),
                );
            }

            if Self::check_shortcut("scroll_down", &settings, event)
                && content_page.read().article_view_scroll_diff(150.0).is_err()
            {
                Util::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select scroll article view down.".to_owned()),
                );
            }

            if Self::check_shortcut("sidebar_set_read", &settings, event) {
                Util::send(&sender, Action::SetSidebarRead);
            }

            Inhibit(false)
        });
    }

    fn check_shortcut(id: &str, settings: &Arc<RwLock<Settings>>, event: &EventKey) -> bool {
        if let Ok(keybinding) = Keybindings::read_keybinding(id, settings) {
            if let Some(keybinding) = keybinding {
                let (keyval, modifier) = gtk::accelerator_parse(&keybinding);

                if gdk::keyval_to_lower(keyval) == gdk::keyval_to_lower(event.get_keyval()) {
                    if modifier.is_empty() {
                        if Keybindings::clean_modifier(event.get_state()).is_empty() {
                            return true;
                        }
                    } else if event.get_state().contains(modifier) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn load_css(provider: &GtkHandle<CssProvider>) {
        let screen = gdk::Screen::get_default().expect(GTK_CSS_ERROR);

        // remove old style provider
        StyleContext::remove_provider_for_screen(&screen, &*provider.borrow());

        // setup new style provider
        let style_sheet = if let Some(settings) = GtkSettings::get_default() {
            if settings.get_property_gtk_application_prefer_dark_theme() {
                "app_dark"
            } else {
                "app"
            }
        } else {
            "app"
        };
        let css_data = Resources::get(&format!("css/{}.css", style_sheet)).expect(GTK_RESOURCE_FILE_ERROR);
        *provider.borrow_mut() = CssProvider::new();
        CssProvider::load_from_data(&*provider.borrow(), css_data.as_ref()).expect(GTK_CSS_ERROR);

        // apply new style provider
        StyleContext::add_provider_for_screen(&screen, &*provider.borrow(), 600);
    }

    pub fn init(&self, news_flash: &Arc<RwLock<Option<NewsFlash>>>, thread_pool: ThreadPool) {
        self.stack.set_visible_child_name(CONTENT_PAGE);
        self.header_stack.set_visible_child_name(CONTENT_PAGE);

        let id = news_flash.read().as_ref().map(|n| n.id());
        let user_name = news_flash.read().as_ref().map(|n| n.user_name());
        if let Some(Some(id)) = id {
            if let Some(user_name) = user_name {
                if self.content_page.read().set_service(&id, user_name).is_err() {
                    Util::send(
                        &self.sender,
                        Action::ErrorSimpleMessage("Failed to set sidebar service logo.".to_owned()),
                    );
                }

                // try to fill content page with data
                self.content_page
                    .write()
                    .update_sidebar(&news_flash, &self.state, &self.undo_bar, thread_pool.clone());
                self.content_page
                    .write()
                    .update_article_list(&news_flash, &self.state, &self.undo_bar, thread_pool);
                return;
            }
        } else {
            warn!("No valid backend ID");
        }

        // in case of failure show 'welcome page'
        self.stack.set_visible_child_name("welcome");
        self.header_stack.set_visible_child_name("welcome");
    }

    pub fn show_error_simple_message(&self, msg: &str) {
        self.error_bar.simple_message(msg);
    }

    pub fn show_error(&self, msg: &str, error: NewsFlashError) {
        self.error_bar.news_flash_error(msg, error);
    }

    pub fn show_undo_bar(&self, action: UndoActionModel) {
        let select_all_button = match self.content_page.read().sidebar_get_selection() {
            SidebarSelection::All => false,
            SidebarSelection::Cateogry((selected_id, _label)) => match &action {
                UndoActionModel::DeleteCategory((delete_id, _label)) => &selected_id == delete_id,
                _ => false,
            },
            SidebarSelection::Feed((selected_id, _label)) => match &action {
                UndoActionModel::DeleteFeed((delete_id, _label)) => &selected_id == delete_id,
                _ => false,
            },
            SidebarSelection::Tag((selected_id, _label)) => match &action {
                UndoActionModel::DeleteTag((delete_id, _label)) => &selected_id == delete_id,
                _ => false,
            },
        };
        if select_all_button {
            self.state.write().set_sidebar_selection(SidebarSelection::All);
            self.content_page.read().sidebar_select_all_button_no_update();
        }

        self.undo_bar.add_action(action);
    }

    pub fn show_welcome_page(&self) {
        self.header_stack.set_visible_child_name("welcome");
        self.password_login_page.reset();
        self.oauth_login_page.reset();
        self.stack.set_transition_type(StackTransitionType::SlideRight);
        self.stack.set_visible_child_name("welcome");
    }

    pub fn show_password_login_page(&self, plugin_id: &PluginID) {
        if let Some(service_meta) = NewsFlash::list_backends().get(plugin_id) {
            if let Ok(()) = self.password_login_page.set_service(service_meta.clone()) {
                self.header_stack.set_visible_child_name("login");
                self.stack.set_transition_type(StackTransitionType::SlideLeft);
                self.stack.set_visible_child_name("password_login");
            }
        }
    }

    pub fn show_oauth_login_page(&self, plugin_id: &PluginID) {
        if let Some(service_meta) = NewsFlash::list_backends().get(plugin_id) {
            if let Ok(()) = self.oauth_login_page.set_service(service_meta.clone()) {
                self.header_stack.set_visible_child_name("login");
                self.stack.set_transition_type(StackTransitionType::SlideLeft);
                self.stack.set_visible_child_name("oauth_login");
            }
        }
    }

    pub fn show_content_page(&self, plugin_id: &PluginID, news_flash: &RwLock<Option<NewsFlash>>) {
        if let Some(news_flash) = news_flash.read().as_ref() {
            let user_name: Option<String> = news_flash.user_name();
            self.stack.set_transition_type(StackTransitionType::SlideLeft);
            self.stack.set_visible_child_name("content");
            self.header_stack.set_visible_child_name("content");

            Util::send(&self.sender, Action::UpdateSidebar);

            if self.content_page.read().set_service(&plugin_id, user_name).is_err() {
                Util::send(
                    &self.sender,
                    Action::ErrorSimpleMessage("Failed to set service.".to_owned()),
                );
            }
        }
    }

    pub fn update_sidebar(&self, news_flash: &Arc<RwLock<Option<NewsFlash>>>, thread_pool: ThreadPool) {
        self.content_page
            .write()
            .update_sidebar(news_flash, &self.state, &self.undo_bar, thread_pool);
    }

    pub fn update_article_list(&self, news_flash: &Arc<RwLock<Option<NewsFlash>>>, thread_pool: ThreadPool) {
        self.content_page
            .write()
            .update_article_list(news_flash, &self.state, &self.undo_bar, thread_pool);
    }

    pub fn load_more_articles(&self, news_flash: &Arc<RwLock<Option<NewsFlash>>>, thread_pool: ThreadPool) {
        self.content_page
            .write()
            .load_more_articles(&news_flash, &self.state, &self.undo_bar, thread_pool);
    }

    pub fn sidebar_selection(&self, selection: SidebarSelection) {
        self.state.write().set_sidebar_selection(selection);
        self.responsive_layout.state.borrow_mut().minor_leaflet_selected = true;
        self.responsive_layout.process_state_change();
        Util::send(&self.sender, Action::UpdateArticleList);
    }

    pub fn show_article(&self, article_id: ArticleID, news_flash: &RwLock<Option<NewsFlash>>) {
        if let Some(news_flash) = news_flash.read().as_ref() {
            let article = match news_flash.get_fat_article(&article_id) {
                Ok(article) => article,
                Err(error) => {
                    Util::send(&self.sender, Action::Error("Failed to read article.".to_owned(), error));
                    return;
                }
            };
            let (feeds, _) = match news_flash.get_feeds() {
                Ok(res) => res,
                Err(error) => {
                    Util::send(&self.sender, Action::Error("Failed to read feeds.".to_owned(), error));
                    return;
                }
            };
            let feed = match feeds.iter().find(|&f| f.feed_id == article.feed_id) {
                Some(feed) => feed,
                None => {
                    Util::send(
                        &self.sender,
                        Action::ErrorSimpleMessage(format!("Failed to find feed: '{}'", article.feed_id)),
                    );
                    return;
                }
            };
            self.content_header.show_article(Some(&article));
            self.content_page.read().article_view_show(article, feed);

            self.responsive_layout.state.borrow_mut().major_leaflet_selected = true;
            self.responsive_layout.process_state_change();
        }
    }

    pub fn set_headerbar_selection(&self, new_selection: HeaderSelection) {
        let old_selection = self.state.read().get_header_selection().clone();
        self.state.write().set_header_selection(new_selection.clone());
        match new_selection {
            HeaderSelection::All => self.content_header.select_all_button(),
            HeaderSelection::Unread => self.content_header.select_unread_button(),
            HeaderSelection::Marked => self.content_header.select_marked_button(),
        };
        Util::send(&self.sender, Action::UpdateArticleList);

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
            Util::send(&self.sender, Action::UpdateSidebar);
        }
    }

    pub fn set_search_term(&self, search_term: String) {
        if search_term.is_empty() {
            self.state.write().set_search_term(None);
        } else {
            self.state.write().set_search_term(Some(search_term));
        }

        Util::send(&self.sender, Action::UpdateArticleList);
    }

    pub fn set_sidebar_read(&self, news_flash: &Arc<RwLock<Option<NewsFlash>>>, threadpool: ThreadPool) {
        let sidebar_selection = self.state.read().get_sidebar_selection().clone();

        match sidebar_selection {
            SidebarSelection::All => {
                let (sender, receiver) = oneshot::channel::<Result<(), NewsFlashError>>();

                let news_flash = news_flash.clone();
                let thread_future = async move {
                    let news_flash = news_flash.clone();
                    let future = async move {
                        if let Some(news_flash) = news_flash.read().as_ref() {
                            sender.send(news_flash.set_all_read().await).unwrap();
                        }
                    };
                    Runtime::new().expect(RUNTIME_ERROR).block_on(future);
                };

                let sender = self.sender.clone();
                let content_header = self.content_header.clone();
                let glib_future = receiver.map(move |res| {
                    content_header.finish_mark_all_read();
                    res.map(|result| match result {
                        Ok(_) => {}
                        Err(error) => {
                            let message = "Failed to mark all read".to_owned();
                            error!("{}", message);
                            Util::send(&sender, Action::Error(message, error));
                        }
                    })
                    .unwrap();
                    Util::send(&sender, Action::UpdateArticleHeader);
                    Util::send(&sender, Action::UpdateArticleList);
                    Util::send(&sender, Action::UpdateSidebar);
                });

                threadpool.spawn_ok(thread_future);
                Util::glib_spawn_future(glib_future);
            }
            SidebarSelection::Cateogry((category_id, _title)) => {
                let (sender, receiver) = oneshot::channel::<Result<(), NewsFlashError>>();

                let news_flash = news_flash.clone();
                let category_id_vec = vec![category_id.clone()];
                let thread_future = async move {
                    if let Some(news_flash) = news_flash.read().as_ref() {
                        sender
                            .send(
                                Runtime::new()
                                    .expect(RUNTIME_ERROR)
                                    .block_on(news_flash.set_category_read(&category_id_vec)),
                            )
                            .unwrap();
                    }
                };

                let sender = self.sender.clone();
                let content_header = self.content_header.clone();
                let glib_future = receiver.map(move |res| {
                    content_header.finish_mark_all_read();
                    res.map(|result| match result {
                        Ok(_) => {}
                        Err(error) => {
                            let message = "Failed to mark all read".to_owned();
                            error!("{}", message);
                            Util::send(&sender, Action::Error(message, error));
                        }
                    })
                    .unwrap();
                    Util::send(&sender, Action::UpdateArticleHeader);
                    Util::send(&sender, Action::UpdateArticleList);
                    Util::send(&sender, Action::UpdateSidebar);
                });

                threadpool.spawn_ok(thread_future);
                Util::glib_spawn_future(glib_future);
            }
            SidebarSelection::Feed((feed_id, _title)) => {
                let (sender, receiver) = oneshot::channel::<Result<(), NewsFlashError>>();

                let news_flash = news_flash.clone();
                let feed_id_vec = vec![feed_id.clone()];
                let thread_future = async move {
                    if let Some(news_flash) = news_flash.read().as_ref() {
                        sender
                            .send(
                                Runtime::new()
                                    .expect(RUNTIME_ERROR)
                                    .block_on(news_flash.set_feed_read(&feed_id_vec)),
                            )
                            .unwrap();
                    }
                };

                let sender = self.sender.clone();
                let content_header = self.content_header.clone();
                let glib_future = receiver.map(move |res| {
                    content_header.finish_mark_all_read();
                    res.map(|result| match result {
                        Ok(_) => {}
                        Err(error) => {
                            let message = "Failed to mark all read".to_owned();
                            error!("{}", message);
                            Util::send(&sender, Action::Error(message, error));
                        }
                    })
                    .unwrap();
                    Util::send(&sender, Action::UpdateArticleHeader);
                    Util::send(&sender, Action::UpdateArticleList);
                    Util::send(&sender, Action::UpdateSidebar);
                });

                threadpool.spawn_ok(thread_future);
                Util::glib_spawn_future(glib_future);
            }
            SidebarSelection::Tag((tag_id, _title)) => {
                let (sender, receiver) = oneshot::channel::<Result<(), NewsFlashError>>();

                let news_flash = news_flash.clone();
                let tag_id_vec = vec![tag_id.clone()];
                let thread_future = async move {
                    if let Some(news_flash) = news_flash.read().as_ref() {
                        sender
                            .send(
                                Runtime::new()
                                    .expect(RUNTIME_ERROR)
                                    .block_on(news_flash.set_tag_read(&tag_id_vec)),
                            )
                            .unwrap();
                    }
                };

                let sender = self.sender.clone();
                let content_header = self.content_header.clone();
                let glib_future = receiver.map(move |res| {
                    content_header.finish_mark_all_read();
                    res.map(|result| match result {
                        Ok(_) => {}
                        Err(error) => {
                            let message = "Failed to mark all read".to_owned();
                            error!("{}", message);
                            Util::send(&sender, Action::Error(message, error));
                        }
                    })
                    .unwrap();
                    Util::send(&sender, Action::UpdateArticleHeader);
                    Util::send(&sender, Action::UpdateArticleList);
                    Util::send(&sender, Action::UpdateSidebar);
                });

                threadpool.spawn_ok(thread_future);
                Util::glib_spawn_future(glib_future);
            }
        }
    }

    pub fn update_article_header(&self, news_flash: &Arc<RwLock<Option<NewsFlash>>>) {
        let visible_article = self.content_page.read().article_view_visible_article();
        if let Some(visible_article) = visible_article {
            if let Some(news_flash) = news_flash.read().as_ref() {
                if let Ok(visible_article) = news_flash.get_fat_article(&visible_article.article_id) {
                    self.content_header.show_article(Some(&visible_article));
                    self.content_page
                        .read()
                        .article_view_update_visible_article(Some(visible_article.unread), None);
                }
            }
        }
    }

    pub fn execute_pending_undoable_action(&self) {
        self.undo_bar.execute_pending_action()
    }
}
