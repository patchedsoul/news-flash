use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;

use gio::{ApplicationExt, ApplicationExtManual, Notification, NotificationPriority, ThemedIcon};
use glib::{futures::FutureExt, translate::ToGlib, Receiver, Sender};
use gtk::{Application, Continue, GtkApplicationExt, GtkWindowExt, GtkWindowExtManual, WidgetExt};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use news_flash::models::{ArticleID, LoginData, PluginID};
use news_flash::{NewsFlash, NewsFlashError};
use parking_lot::RwLock;

use crate::about_dialog::NewsFlashAbout;
use crate::article_list::{MarkUpdate, ReadUpdate};
use crate::config::APP_ID;
use crate::content_page::HeaderSelection;
use crate::main_window::MainWindow;
use crate::settings::{NewsFlashShortcutWindow, Settings, SettingsDialog};
use crate::sidebar::models::SidebarSelection;
use crate::undo_bar::UndoActionModel;
use crate::util::GtkUtil;

lazy_static! {
    pub static ref DATA_DIR: PathBuf = glib::get_user_config_dir()
        .expect("Failed to find the config dir")
        .join("news-flash");
}

#[derive(Debug, Clone)]
pub struct NotificationCounts {
    pub new: i64,
    pub unread: i64,
}

#[derive(Debug)]
pub enum Action {
    ShowNotification(NotificationCounts),
    ErrorSimpleMessage(String),
    Error(String, NewsFlashError),
    UndoableAction(UndoActionModel),
    ShowWelcomePage,
    ShowContentPage(PluginID),
    ShowPasswordLogin(PluginID),
    ShowOauthLogin(PluginID),
    ShowSettingsWindow,
    ShowShortcutWindow,
    ShowAboutWindow,
    Login(LoginData),
    ScheduleSync,
    Sync,
    MarkArticleRead(ReadUpdate),
    MarkArticle(MarkUpdate),
    ToggleArticleRead,
    ToggleArticleMarked,
    UpdateSidebar,
    UpdateArticleList,
    LoadMoreArticles,
    SidebarSelection(SidebarSelection),
    HeaderSelection(HeaderSelection),
    ShowArticle(ArticleID),
    RedrawArticle,
    CloseArticle,
    SearchTerm(String),
    SetSidebarRead,
    Quit,
}
pub struct App {
    application: gtk::Application,
    window: MainWindow,
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,
    news_flash: RwLock<Option<NewsFlash>>,
    settings: Rc<RwLock<Settings>>,
    sync_source_id: RwLock<Option<u32>>,
}

impl App {
    pub fn new() -> Rc<Self> {
        let application =
            Application::new(Some(APP_ID), gio::ApplicationFlags::empty()).expect("Initialization gtk-app failed");

        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let news_flash = RwLock::new(None);
        let settings = Rc::new(RwLock::new(Settings::open().expect("Failed to access settings file")));
        let window = MainWindow::new(&settings, sender.clone());

        let app = Rc::new(Self {
            application,
            window,
            sender,
            receiver,
            news_flash,
            settings,
            sync_source_id: RwLock::new(None),
        });

        app.setup_signals();

        if let Ok(news_flash_lib) = NewsFlash::try_load(&crate::app::DATA_DIR) {
            info!("Successful load from config");
            app.news_flash.write().replace(news_flash_lib);
            GtkUtil::send(&app.sender, Action::ScheduleSync);
        } else {
            warn!("No account configured");
        }

        app.window.init(&app.news_flash);

        app
    }

    fn setup_signals(&self) {
        self.application.connect_startup(|_app| {
            debug!("startup");
        });

        let window = self.window.widget.clone();
        self.application.connect_activate(move |app| {
            debug!("activate");
            app.add_window(&window);
            window.show_all();
            window.present();
        });
    }

    pub fn run(&self, app: Rc<Self>) {
        debug!("run");
        let a = app.clone();
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| a.process_action(action));

        let args: Vec<String> = env::args().collect();
        self.application.run(&args);
    }

    fn process_action(&self, action: Action) -> glib::Continue {
        match action {
            Action::ShowNotification(counts) => self.show_notification(counts),
            Action::ErrorSimpleMessage(msg) => self.window.show_error_simple_message(&msg),
            Action::Error(msg, error) => self.window.show_error(&msg, error),
            Action::UndoableAction(action) => self.window.show_undo_bar(action),
            Action::ShowWelcomePage => self.window.show_welcome_page(),
            Action::ShowContentPage(plugin_id) => self.window.show_content_page(&plugin_id, &self.news_flash),
            Action::ShowPasswordLogin(plugin_id) => self.window.show_password_login_page(&plugin_id),
            Action::ShowOauthLogin(plugin_id) => self.window.show_oauth_login_page(&plugin_id),
            Action::ShowSettingsWindow => self.spawn_settings_window(),
            Action::ShowShortcutWindow => self.spawn_shortcut_window(),
            Action::ShowAboutWindow => self.spawn_about_window(),
            Action::Login(data) => self.login(data),
            Action::ScheduleSync => self.schedule_sync(),
            Action::Sync => self.sync(),
            Action::MarkArticleRead(update) => self.mark_article_read(update),
            Action::MarkArticle(update) => self.mark_article(update),
            Action::ToggleArticleRead => self.toggle_article_read(),
            Action::ToggleArticleMarked => self.toggle_article_marked(),
            Action::UpdateSidebar => self.window.update_sidebar(&self.news_flash),
            Action::UpdateArticleList => self.window.update_article_list(&self.news_flash),
            Action::LoadMoreArticles => self.window.load_more_articles(&self.news_flash),
            Action::SidebarSelection(selection) => self.window.sidebar_selection(selection),
            Action::HeaderSelection(selection) => self.window.set_headerbar_selection(selection),
            Action::ShowArticle(article_id) => self.window.show_article(article_id, &self.news_flash),
            Action::RedrawArticle => self.window.content_page.borrow_mut().article_view_redraw(),
            Action::CloseArticle => {
                self.window.content_page.borrow().article_view_close();
                self.window.content_header.show_article(None);
            }
            Action::SearchTerm(search_term) => self.window.set_search_term(search_term),
            Action::SetSidebarRead => self.window.set_sidebar_read(&self.news_flash),
            Action::Quit => self.quit(),
        }
        glib::Continue(true)
    }

    fn show_notification(&self, counts: NotificationCounts) {
        if counts.new > 0 && counts.unread > 0 {
            let summary = "New Articles";

            let message = if counts.new == 1 {
                format!("There is 1 new article ({} unread)", counts.unread)
            } else {
                format!("There are {} new articles ({} unread)", counts.new, counts.unread)
            };

            let notification = Notification::new(summary);
            notification.set_body(Some(&message));
            notification.set_priority(NotificationPriority::Normal);
            notification.set_icon(&ThemedIcon::new(APP_ID));

            self.application
                .send_notification(Some("newsflash_sync"), &notification);
        }
    }

    fn login(&self, data: LoginData) {
        let id = match &data {
            LoginData::OAuth(oauth) => oauth.id.clone(),
            LoginData::Password(pass) => pass.id.clone(),
            LoginData::None(id) => id.clone(),
        };
        let mut news_flash_lib = match NewsFlash::new(&DATA_DIR, &id) {
            Ok(news_flash) => news_flash,
            Err(error) => {
                match &data {
                    LoginData::OAuth(_) => self.window.oauth_logn_page.show_error(error),
                    LoginData::Password(_) => self.window.password_login_page.show_error(error),
                    LoginData::None(_) => {}
                }
                return;
            }
        };

        let login_result = GtkUtil::block_on_future(news_flash_lib.login(data.clone()));
        match login_result {
            Ok(()) => {
                // create main obj
                self.news_flash.write().replace(news_flash_lib);

                // show content page
                GtkUtil::send(&self.sender, Action::ShowContentPage(id));
            }
            Err(error) => {
                error!("Login failed! Plguin: {}, Error: {}", id, error);
                match data {
                    LoginData::OAuth(_) => {
                        self.window.oauth_logn_page.show_error(error);
                    }
                    LoginData::Password(_) => {
                        self.window.password_login_page.show_error(error);
                    }
                    LoginData::None(_) => {
                        // NOTHING
                    }
                }
            }
        }
    }

    fn schedule_sync(&self) {
        GtkUtil::remove_source(*self.sync_source_id.read());
        let sync_interval = self.settings.read().get_sync_interval();
        if let Some(sync_interval) = sync_interval.to_seconds() {
            let sender = self.sender.clone();
            self.sync_source_id.write().replace(
                gtk::timeout_add_seconds(sync_interval, move || {
                    GtkUtil::send(&sender, Action::Sync);
                    Continue(true)
                })
                .to_glib(),
            );
        } else {
            self.sync_source_id.write().take();
        }
    }

    fn sync(&self) {
        if let Some(news_flash) = self.news_flash.read().as_ref() {
            let sync_result = GtkUtil::block_on_future(news_flash.sync());
            let unread_count = match news_flash.unread_count_all() {
                Ok(unread_count) => unread_count,
                Err(_) => 0,
            };
            match sync_result {
                Ok(new_article_count) => {
                    self.window.content_header.finish_sync();
                    GtkUtil::send(&self.sender, Action::UpdateSidebar);
                    GtkUtil::send(&self.sender, Action::UpdateArticleList);
                    let counts = NotificationCounts {
                        new: new_article_count,
                        unread: unread_count,
                    };
                    GtkUtil::send(&self.sender, Action::ShowNotification(counts));
                }
                Err(error) => {
                    self.window.content_header.finish_sync();
                    GtkUtil::send(&self.sender, Action::Error("Failed to sync.".to_owned(), error));
                }
            }
        }
    }

    fn mark_article_read(&self, update: ReadUpdate) {
        if let Some(news_flash) = self.news_flash.write().as_mut() {
            let article_id_vec = vec![update.article_id.clone()];
            let future = news_flash
                .set_article_read(&article_id_vec, update.read)
                .map(|result| match result {
                    Ok(_) => {}
                    Err(error) => {
                        let message = format!("Failed to mark article read: '{}'", update.article_id);
                        error!("{}", message);
                        GtkUtil::send(&self.sender, Action::Error(message, error));
                    }
                });
            GtkUtil::block_on_future(future);
        } else {
            let message = "Failed to borrow NewsFlash.".to_owned();
            error!("{}", message);
            GtkUtil::send(&self.sender, Action::ErrorSimpleMessage(message));
        }

        GtkUtil::send(&self.sender, Action::UpdateSidebar);
        let visible_article = self.window.content_page.borrow().article_view_visible_article();
        if let Some(visible_article) = visible_article {
            if visible_article.article_id == update.article_id {
                let mut visible_article = visible_article.clone();
                visible_article.unread = update.read;
                self.window.content_header.show_article(Some(&visible_article));
                self.window
                    .content_page
                    .borrow_mut()
                    .article_view_update_visible_article(Some(visible_article.unread), None);
            }
        }
    }

    fn mark_article(&self, update: MarkUpdate) {
        if let Some(news_flash) = self.news_flash.write().as_mut() {
            let article_id_vec = vec![update.article_id.clone()];
            let future = news_flash
                .set_article_marked(&article_id_vec, update.marked)
                .map(|result| match result {
                    Ok(_) => {}
                    Err(error) => {
                        let message = format!("Failed to star article: '{}'", update.article_id);
                        error!("{}", message);
                        GtkUtil::send(&self.sender, Action::Error(message, error));
                    }
                });
            GtkUtil::block_on_future(future);
        } else {
            let message = "Failed to borrow NewsFlash.".to_owned();
            error!("{}", message);
            GtkUtil::send(&self.sender, Action::ErrorSimpleMessage(message));
        }

        GtkUtil::send(&self.sender, Action::UpdateSidebar);
        let visible_article = self.window.content_page.borrow().article_view_visible_article();
        if let Some(visible_article) = visible_article {
            if visible_article.article_id == update.article_id {
                let mut visible_article = visible_article.clone();
                visible_article.marked = update.marked;
                self.window.content_header.show_article(Some(&visible_article));
                self.window
                    .content_page
                    .borrow_mut()
                    .article_view_update_visible_article(None, Some(visible_article.marked));
            }
        }
    }

    fn toggle_article_read(&self) {
        let visible_article = self.window.content_page.borrow().article_view_visible_article();
        if let Some(visible_article) = visible_article {
            let update = ReadUpdate {
                article_id: visible_article.article_id.clone(),
                read: visible_article.unread.invert(),
            };
            GtkUtil::send(&self.sender, Action::MarkArticleRead(update));
            GtkUtil::send(&self.sender, Action::UpdateArticleList);
        }
    }

    fn toggle_article_marked(&self) {
        let visible_article = self.window.content_page.borrow().article_view_visible_article();
        if let Some(visible_article) = visible_article {
            let update = MarkUpdate {
                article_id: visible_article.article_id.clone(),
                marked: visible_article.marked.invert(),
            };

            GtkUtil::send(&self.sender, Action::MarkArticle(update));
            GtkUtil::send(&self.sender, Action::UpdateArticleList);
        }
    }

    fn spawn_shortcut_window(&self) {
        let dialog = NewsFlashShortcutWindow::new(&self.window.widget, &*self.settings.read());
        dialog.widget.present();
    }

    fn spawn_about_window(&self) {
        let dialog = NewsFlashAbout::new(&self.window.widget);
        dialog.widget.present();
    }

    fn spawn_settings_window(&self) {
        let dialog = SettingsDialog::new(&self.window.widget, &self.sender, &self.settings);
        dialog.widget.present();
    }

    fn quit(&self) {
        // FIXME: check for ongoing sync
        self.window.widget.close();
        self.application.quit();
    }
}
