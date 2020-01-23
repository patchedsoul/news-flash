use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time;

use futures::channel::oneshot::{self, Sender as OneShotSender};
use futures::executor::ThreadPool;
use gio::{ApplicationExt, ApplicationExtManual, Notification, NotificationPriority, ThemedIcon};
use glib::{futures::FutureExt, translate::ToGlib, Receiver, Sender};
use gtk::{
    Application, ButtonExt, Continue, DialogExt, EntryExt, FileChooserAction, FileChooserDialog, FileChooserExt,
    FileFilter, GtkApplicationExt, GtkWindowExt, GtkWindowExtManual, ResponseType, WidgetExt,
};
use lazy_static::lazy_static;
use log::{error, info, warn};
use news_flash::models::{ArticleID, Category, CategoryID, FavIcon, Feed, FeedID, LoginData, PluginID, TagID, Url};
use news_flash::{NewsFlash, NewsFlashError};
use parking_lot::RwLock;
use tokio::runtime::Runtime;

use crate::about_dialog::NewsFlashAbout;
use crate::add_dialog::{AddCategory, AddPopover};
use crate::article_list::{MarkUpdate, ReadUpdate};
use crate::article_view::ArticleView;
use crate::config::APP_ID;
use crate::content_page::HeaderSelection;
use crate::main_window::MainWindow;
use crate::rename_dialog::RenameDialog;
use crate::settings::{NewsFlashShortcutWindow, Settings, SettingsDialog};
use crate::sidebar::{models::SidebarSelection, FeedListDndAction};
use crate::undo_bar::UndoActionModel;
use crate::util::{FileUtil, GtkUtil, Util, CHANNEL_ERROR, RUNTIME_ERROR};

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
    LoadFavIcon((Feed, OneShotSender<Option<FavIcon>>)),
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
    SidebarSelectNext,
    SidebarSelectPrev,
    HeaderSelection(HeaderSelection),
    UpdateArticleHeader,
    ShowArticle(ArticleID),
    RedrawArticle,
    CloseArticle,
    SearchTerm(String),
    SetSidebarRead,
    AddFeedDialog,
    AddFeed((Url, Option<String>, Option<AddCategory>)),
    RenameFeedDialog(FeedID),
    RenameFeed((Feed, String)),
    RenameCategoryDialog(CategoryID),
    RenameCategory((Category, String)),
    DeleteSidebarSelection,
    DeleteFeed(FeedID),
    DeleteCategory(CategoryID),
    DeleteTag(TagID),
    DragAndDrop(FeedListDndAction),
    ExportArticle,
    ExportOpml,
    QueueQuit,
    ForceQuit,
}
pub struct App {
    application: gtk::Application,
    window: MainWindow,
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,
    news_flash: Arc<RwLock<Option<NewsFlash>>>,
    settings: Arc<RwLock<Settings>>,
    sync_source_id: RwLock<Option<u32>>,
    threadpool: ThreadPool,
}

impl App {
    pub fn new() -> Rc<Self> {
        let application =
            Application::new(Some(APP_ID), gio::ApplicationFlags::empty()).expect("Initialization gtk-app failed");

        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let news_flash = Arc::new(RwLock::new(None));
        let settings = Arc::new(RwLock::new(Settings::open().expect("Failed to access settings file")));
        let window = MainWindow::new(&settings, sender.clone());

        let app = Rc::new(Self {
            application,
            window,
            sender,
            receiver,
            news_flash,
            settings,
            sync_source_id: RwLock::new(None),
            threadpool: ThreadPool::new().unwrap(),
        });

        app.setup_signals();

        if let Ok(news_flash_lib) = NewsFlash::try_load(&crate::app::DATA_DIR) {
            info!("Successful load from config");
            app.news_flash.write().replace(news_flash_lib);
            Util::send(&app.sender, Action::ScheduleSync);
        } else {
            warn!("No account configured");
        }

        app.window.init(&app.news_flash, app.threadpool.clone());

        app
    }

    fn setup_signals(&self) {
        self.application.connect_startup(|_app| {});

        let window = self.window.widget.clone();
        self.application.connect_activate(move |app| {
            app.add_window(&window);
            window.show_all();
            window.present();
        });
    }

    pub fn run(&self, app: Rc<Self>) {
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
            Action::LoadFavIcon((feed, sender)) => self.load_favicon(feed, sender),
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
            Action::UpdateArticleList => self.window.update_article_list(&self.news_flash, self.threadpool.clone()),
            Action::LoadMoreArticles => self.window.load_more_articles(&self.news_flash, self.threadpool.clone()),
            Action::SidebarSelection(selection) => self.window.sidebar_selection(selection),
            Action::SidebarSelectNext => self.window.content_page.read().select_next_article(),
            Action::SidebarSelectPrev => self.window.content_page.read().select_prev_article(),
            Action::HeaderSelection(selection) => self.window.set_headerbar_selection(selection),
            Action::UpdateArticleHeader => self.window.update_article_header(&self.news_flash),
            Action::ShowArticle(article_id) => self.window.show_article(article_id, &self.news_flash),
            Action::RedrawArticle => self.window.content_page.read().article_view_redraw(),
            Action::CloseArticle => {
                self.window.content_page.read().article_view_close();
                self.window.content_header.show_article(None);
            }
            Action::SearchTerm(search_term) => self.window.set_search_term(search_term),
            Action::SetSidebarRead => self.window.set_sidebar_read(&self.news_flash, self.threadpool.clone()),
            Action::AddFeedDialog => self.add_feed_dialog(),
            Action::AddFeed((url, title, category)) => self.add_feed(url, title, category),
            Action::RenameFeedDialog(feed_id) => self.rename_feed_dialog(feed_id),
            Action::RenameFeed((feed, new_title)) => self.rename_feed(feed, new_title),
            Action::RenameCategoryDialog(category_id) => self.rename_category_dialog(category_id),
            Action::RenameCategory((category, new_title)) => self.rename_category(category, new_title),
            Action::DeleteSidebarSelection => self.delete_selection(),
            Action::DeleteFeed(feed_id) => self.delete_feed(feed_id),
            Action::DeleteCategory(category_id) => self.delete_category(category_id),
            Action::DeleteTag(tag_id) => self.delete_tag(tag_id),
            Action::DragAndDrop(action) => self.drag_and_drop(action),
            Action::ExportArticle => self.export_article(),
            Action::ExportOpml => self.export_opml(),
            Action::QueueQuit => self.queue_quit(),
            Action::ForceQuit => self.force_quit(),
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
                    LoginData::OAuth(_) => self.window.oauth_login_page.show_error(error),
                    LoginData::Password(_) => self.window.password_login_page.show_error(error),
                    LoginData::None(_) => {}
                }
                return;
            }
        };

        let (sender, receiver) = oneshot::channel::<Result<(), NewsFlashError>>();

        let news_flash = self.news_flash.clone();
        let global_sender = self.sender.clone();
        let data_clone = data.clone();
        let thread_future = async move {
            let result = Runtime::new()
                .expect(RUNTIME_ERROR)
                .block_on(news_flash_lib.login(data_clone));
            match &result {
                Ok(()) => {
                    // create main obj
                    news_flash.write().replace(news_flash_lib);
                    // show content page
                    Util::send(&global_sender, Action::ShowContentPage(id));
                }
                Err(error) => {
                    error!("Login failed! Plguin: {}, Error: {}", id, error);
                }
            }
            sender.send(result).expect(CHANNEL_ERROR);
        };

        let oauth_login_page = self.window.oauth_login_page.clone();
        let password_login_page = self.window.password_login_page.clone();
        let glib_future = receiver.map(move |res| {
            if let Ok(Err(error)) = res {
                match data {
                    LoginData::OAuth(_) => {
                        oauth_login_page.show_error(error);
                    }
                    LoginData::Password(_) => {
                        password_login_page.show_error(error);
                    }
                    LoginData::None(_) => {
                        // NOTHING
                    }
                }
            }
        });

        self.threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn schedule_sync(&self) {
        GtkUtil::remove_source(*self.sync_source_id.read());
        let sync_interval = self.settings.read().get_sync_interval();
        if let Some(sync_interval) = sync_interval.to_seconds() {
            let sender = self.sender.clone();
            self.sync_source_id.write().replace(
                gtk::timeout_add_seconds(sync_interval, move || {
                    Util::send(&sender, Action::Sync);
                    Continue(true)
                })
                .to_glib(),
            );
        } else {
            self.sync_source_id.write().take();
        }
    }

    fn sync(&self) {
        let (sender, receiver) = oneshot::channel::<Result<i64, NewsFlashError>>();

        let news_flash = self.news_flash.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let result = Runtime::new().expect(RUNTIME_ERROR).block_on(news_flash.sync());
                sender.send(result).expect(CHANNEL_ERROR);
            }
        };

        let news_flash = self.news_flash.clone();
        let sender = self.sender.clone();
        let content_header = self.window.content_header.clone();
        let glib_future = receiver.map(move |res| {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let unread_count = match news_flash.unread_count_all() {
                    Ok(unread_count) => unread_count,
                    Err(_) => 0,
                };
                match res {
                    Ok(Ok(new_article_count)) => {
                        content_header.finish_sync();
                        Util::send(&sender, Action::UpdateSidebar);
                        Util::send(&sender, Action::UpdateArticleList);
                        let counts = NotificationCounts {
                            new: new_article_count,
                            unread: unread_count,
                        };
                        Util::send(&sender, Action::ShowNotification(counts));
                    }
                    Ok(Err(error)) => {
                        content_header.finish_sync();
                        Util::send(&sender, Action::Error("Failed to sync.".to_owned(), error));
                    }
                    Err(_) => {}
                }
            }
        });

        self.threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn load_favicon(&self, feed: Feed, oneshot_sender: OneShotSender<Option<FavIcon>>) {
        let news_flash = self.news_flash.clone();
        let global_sender = self.sender.clone();
        let feed = feed.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.write().as_mut() {
                let favicon = match Runtime::new()
                    .expect(RUNTIME_ERROR)
                    .block_on(news_flash.get_icon_info(&feed))
                {
                    Ok(favicon) => Some(favicon),
                    Err(_) => None,
                };
                oneshot_sender.send(favicon).expect(CHANNEL_ERROR);
            } else {
                let message = "Failed to lock NewsFlash.".to_owned();
                error!("{}", message);
                Util::send(&global_sender, Action::ErrorSimpleMessage(message));
            }
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn mark_article_read(&self, update: ReadUpdate) {
        let (sender, receiver) = oneshot::channel::<Result<(), NewsFlashError>>();

        let news_flash = self.news_flash.clone();
        let article_id_vec = vec![update.article_id.clone()];
        let read_status = update.read;
        let global_sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                sender
                    .send(
                        Runtime::new()
                            .expect(RUNTIME_ERROR)
                            .block_on(news_flash.set_article_read(&article_id_vec, read_status)),
                    )
                    .expect(CHANNEL_ERROR);
            } else {
                let message = "Failed to lock NewsFlash.".to_owned();
                error!("{}", message);
                Util::send(&global_sender, Action::ErrorSimpleMessage(message));
            }
        };

        let global_sender = self.sender.clone();
        let content_page = self.window.content_page.clone();
        let content_header = self.window.content_header.clone();
        let glib_future = receiver.map(move |res| {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    let message = format!("Failed to mark article read: '{}'", update.article_id);
                    error!("{}", message);
                    Util::send(&global_sender, Action::Error(message, error));
                }
                Err(error) => {
                    let message = format!("Sender error: {}", error);
                    error!("{}", message);
                    Util::send(&global_sender, Action::ErrorSimpleMessage(message));
                }
            };

            Util::send(&global_sender, Action::UpdateSidebar);
            let visible_article = content_page.read().article_view_visible_article();
            if let Some(visible_article) = visible_article {
                if visible_article.article_id == update.article_id {
                    let mut visible_article = visible_article.clone();
                    visible_article.unread = update.read;
                    content_header.show_article(Some(&visible_article));
                    content_page
                        .read()
                        .article_view_update_visible_article(Some(visible_article.unread), None);
                }
            }
        });

        self.threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn mark_article(&self, update: MarkUpdate) {
        let (sender, receiver) = oneshot::channel::<Result<(), NewsFlashError>>();

        let news_flash = self.news_flash.clone();
        let article_id_vec = vec![update.article_id.clone()];
        let mark_status = update.marked;
        let global_sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                sender
                    .send(
                        Runtime::new()
                            .expect(RUNTIME_ERROR)
                            .block_on(news_flash.set_article_marked(&article_id_vec, mark_status)),
                    )
                    .expect(CHANNEL_ERROR);
            } else {
                let message = "Failed to lock NewsFlash.".to_owned();
                error!("{}", message);
                Util::send(&global_sender, Action::ErrorSimpleMessage(message));
            }
        };

        let global_sender = self.sender.clone();
        let content_page = self.window.content_page.clone();
        let content_header = self.window.content_header.clone();
        let glib_future = receiver.map(move |_res| {
            Util::send(&global_sender, Action::UpdateSidebar);
            let visible_article = content_page.read().article_view_visible_article();
            if let Some(visible_article) = visible_article {
                if visible_article.article_id == update.article_id {
                    let mut visible_article = visible_article.clone();
                    visible_article.marked = update.marked;
                    content_header.show_article(Some(&visible_article));
                    content_page
                        .read()
                        .article_view_update_visible_article(None, Some(visible_article.marked));
                }
            }
        });

        self.threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn toggle_article_read(&self) {
        let visible_article = self.window.content_page.read().article_view_visible_article();
        if let Some(visible_article) = visible_article {
            let update = ReadUpdate {
                article_id: visible_article.article_id.clone(),
                read: visible_article.unread.invert(),
            };
            Util::send(&self.sender, Action::MarkArticleRead(update));
            Util::send(&self.sender, Action::UpdateArticleList);
        }
    }

    fn toggle_article_marked(&self) {
        let visible_article = self.window.content_page.read().article_view_visible_article();
        if let Some(visible_article) = visible_article {
            let update = MarkUpdate {
                article_id: visible_article.article_id.clone(),
                marked: visible_article.marked.invert(),
            };

            Util::send(&self.sender, Action::MarkArticle(update));
            Util::send(&self.sender, Action::UpdateArticleList);
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

    fn add_feed_dialog(&self) {
        if let Some(news_flash) = self.news_flash.read().as_ref() {
            let error_message = "Failed to add feed".to_owned();
            let add_button = self.window.content_page.read().sidebar_get_add_button();

            let categories = match news_flash.get_categories() {
                Ok(categories) => categories,
                Err(error) => {
                    error!("{}", error_message);
                    Util::send(&self.sender, Action::Error(error_message.clone(), error));
                    return;
                }
            };
            let dialog = AddPopover::new(&add_button, categories, self.threadpool.clone());
            let sender = self.sender.clone();
            dialog.add_button().connect_clicked(move |_button| {
                let feed_url = match dialog.get_feed_url() {
                    Some(url) => url,
                    None => {
                        error!("{}: No valid url", error_message);
                        Util::send(&sender, Action::ErrorSimpleMessage(error_message.clone()));
                        return;
                    }
                };
                let feed_title = dialog.get_feed_title();
                let feed_category = dialog.get_category();

                Util::send(&sender, Action::AddFeed((feed_url, feed_title, feed_category)));
                dialog.close();
            });
        }
    }

    fn add_feed(&self, feed_url: Url, title: Option<String>, category: Option<AddCategory>) {
        info!("add feed '{}'", feed_url);
        
        let news_flash = self.news_flash.clone();
        let global_sender = self.sender.clone();
        let thread_future = async move {
            let error_message = "Failed to add feed".to_owned();
            if let Some(news_flash) = news_flash.read().as_ref() {
                let category_id = match category {
                    Some(category) => match category {
                        AddCategory::New(category_title) => {
                            let add_category_future = news_flash.add_category(&category_title, None, None);
                            let category = match Runtime::new().expect(RUNTIME_ERROR).block_on(add_category_future) {
                                Ok(category) => category,
                                Err(error) => {
                                    error!("{}: Can't add Category", error_message);
                                    Util::send(&global_sender, Action::Error(error_message.clone(), error));
                                    return;
                                }
                            };
                            Some(category.category_id)
                        }
                        AddCategory::Existing(category_id) => Some(category_id),
                    },
                    None => None,
                };

                let add_feed_future = news_flash
                    .add_feed(&feed_url, title, category_id)
                    .map(|result| match result {
                        Ok(_) => {}
                        Err(error) => {
                            error!("{}: Can't add Feed", error_message);
                            Util::send(&global_sender, Action::Error(error_message.clone(), error));
                        }
                    });
                Runtime::new().expect(RUNTIME_ERROR).block_on(add_feed_future);
                Util::send(&global_sender, Action::UpdateSidebar);
            } else {
                let message = "Failed to lock NewsFlash.".to_owned();
                error!("{}", message);
                Util::send(&global_sender, Action::ErrorSimpleMessage(message));
            }
        };
        self.threadpool.spawn_ok(thread_future);
    }

    fn rename_feed_dialog(&self, feed_id: FeedID) {
        if let Some(news_flash) = self.news_flash.read().as_ref() {
            let (feeds, _mappings) = match news_flash.get_feeds() {
                Ok(result) => result,
                Err(error) => {
                    let message = "Failed to laod list of feeds.".to_owned();
                    Util::send(&self.sender, Action::Error(message, error));
                    return;
                }
            };

            let feed = match feeds.iter().find(|f| f.feed_id == feed_id).cloned() {
                Some(feed) => feed,
                None => {
                    let message = format!("Failed to find feed '{}'", feed_id);
                    Util::send(&self.sender, Action::ErrorSimpleMessage(message));
                    return;
                }
            };

            let dialog = RenameDialog::new(
                &self.window.widget,
                &SidebarSelection::Feed((feed_id, feed.label.clone())),
            );
            let rename_button = dialog.rename_button();
            let rename_entry = dialog.rename_entry();
            let rename_dialog = dialog.dialog();
            let sender = self.sender.clone();
            rename_button.connect_clicked(move |_button| {
                let new_label = match rename_entry.get_text().map(|label| label.to_owned()) {
                    Some(label) => label,
                    None => {
                        Util::send(
                            &sender,
                            Action::ErrorSimpleMessage("No valid title to rename feed.".to_owned()),
                        );
                        rename_dialog.emit_close();
                        return;
                    }
                };

                let feed = feed.clone();
                Util::send(&sender, Action::RenameFeed((feed, new_label)));
                rename_dialog.emit_close();
            });
        }
    }

    fn rename_feed(&self, feed: Feed, new_title: String) {
        let news_flash = self.news_flash.clone();
        let sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                if let Err(error) = Runtime::new()
                    .expect(RUNTIME_ERROR)
                    .block_on(news_flash.rename_feed(&feed, &new_title))
                {
                    Util::send(&sender, Action::Error("Failed to rename feed.".to_owned(), error));
                }
            }

            Util::send(&sender, Action::UpdateArticleList);
            Util::send(&sender, Action::UpdateSidebar);
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn rename_category_dialog(&self, category_id: CategoryID) {
        if let Some(news_flash) = self.news_flash.read().as_ref() {
            let categories = match news_flash.get_categories() {
                Ok(categories) => categories,
                Err(error) => {
                    let message = "Failed to load list of categories.".to_owned();
                    Util::send(&self.sender, Action::Error(message, error));
                    return;
                }
            };

            let category = match categories.iter().find(|c| c.category_id == category_id).cloned() {
                Some(category) => category,
                None => {
                    let message = format!("Failed to find category '{}'", category_id);
                    Util::send(&self.sender, Action::ErrorSimpleMessage(message));
                    return;
                }
            };

            let dialog = RenameDialog::new(
                &self.window.widget,
                &SidebarSelection::Cateogry((category_id, category.label.clone())),
            );

            let rename_button = dialog.rename_button();
            let rename_entry = dialog.rename_entry();
            let rename_dialog = dialog.dialog();
            let sender = self.sender.clone();
            rename_button.connect_clicked(move |_button| {
                let new_label = match rename_entry.get_text().map(|label| label.to_owned()) {
                    Some(label) => label,
                    None => {
                        Util::send(
                            &sender,
                            Action::ErrorSimpleMessage("No valid title to rename feed.".to_owned()),
                        );
                        rename_dialog.emit_close();
                        return;
                    }
                };

                let category = category.clone();
                Util::send(&sender, Action::RenameCategory((category, new_label)));
                rename_dialog.emit_close();
            });
        }
    }

    fn rename_category(&self, category: Category, new_title: String) {
        let news_flash = self.news_flash.clone();
        let sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                if let Err(error) = Runtime::new()
                    .expect(RUNTIME_ERROR)
                    .block_on(news_flash.rename_category(&category, &new_title))
                {
                    Util::send(&sender, Action::Error("Failed to rename category.".to_owned(), error));
                }
            }

            Util::send(&sender, Action::UpdateSidebar);
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn delete_selection(&self) {
        let selection = self.window.content_page.read().sidebar_get_selection();
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
            Util::send(&self.sender, Action::UndoableAction(undo_action));
        }
    }

    fn delete_feed(&self, feed_id: FeedID) {
        let news_flash = self.news_flash.clone();
        let sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let (feeds, _mappings) = match news_flash.get_feeds() {
                    Ok(res) => res,
                    Err(error) => {
                        Util::send(&sender, Action::Error("Failed to delete feed.".to_owned(), error));
                        return;
                    }
                };

                if let Some(feed) = feeds.iter().find(|f| f.feed_id == feed_id).cloned() {
                    info!("delete feed '{}' (id: {})", feed.label, feed.feed_id);
                    if let Err(error) = Runtime::new()
                        .expect(RUNTIME_ERROR)
                        .block_on(news_flash.remove_feed(&feed))
                    {
                        Util::send(&sender, Action::Error("Failed to delete feed.".to_owned(), error));
                    }
                } else {
                    let message = format!("Failed to delete feed: feed with id '{}' not found.", feed_id);
                    Util::send(&sender, Action::ErrorSimpleMessage(message));
                    error!("feed not found: {}", feed_id);
                }
            }
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn delete_category(&self, category_id: CategoryID) {
        let news_flash = self.news_flash.clone();
        let sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let categories = match news_flash.get_categories() {
                    Ok(res) => res,
                    Err(error) => {
                        Util::send(&sender, Action::Error("Failed to delete category.".to_owned(), error));
                        return;
                    }
                };

                if let Some(category) = categories.iter().find(|c| c.category_id == category_id).cloned() {
                    info!("delete category '{}' (id: {})", category.label, category.category_id);
                    if let Err(error) = Runtime::new()
                        .expect(RUNTIME_ERROR)
                        .block_on(news_flash.remove_category(&category, true))
                    {
                        Util::send(&sender, Action::Error("Failed to delete category.".to_owned(), error));
                    }
                } else {
                    let message = format!(
                        "Failed to delete category: category with id '{}' not found.",
                        category_id
                    );
                    Util::send(&sender, Action::ErrorSimpleMessage(message));
                    error!("category not found: {}", category_id);
                }
            }
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn delete_tag(&self, tag_id: TagID) {
        let news_flash = self.news_flash.clone();
        let sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let tags = match news_flash.get_tags() {
                    Ok(res) => res,
                    Err(error) => {
                        Util::send(&sender, Action::Error("Failed to delete tag.".to_owned(), error));
                        return;
                    }
                };

                if let Some(tag) = tags.iter().find(|t| t.tag_id == tag_id).cloned() {
                    info!("delete tag '{}' (id: {})", tag.label, tag.tag_id);
                    if let Err(error) = Runtime::new()
                        .expect(RUNTIME_ERROR)
                        .block_on(news_flash.remove_tag(&tag))
                    {
                        Util::send(&sender, Action::Error("Failed to delete tag.".to_owned(), error));
                    }
                } else {
                    let message = format!("Failed to delete tag: tag with id '{}' not found.", tag_id);
                    Util::send(&sender, Action::ErrorSimpleMessage(message));
                    error!("tag not found: {}", tag_id);
                }
            }
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn drag_and_drop(&self, action: FeedListDndAction) {
        let news_flash = self.news_flash.clone();
        let sender = self.sender.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let mut runtime = Runtime::new().expect(RUNTIME_ERROR);
                match action {
                    FeedListDndAction::MoveCategory(category_id, parent_id, _sort_index) => {
                        if let Err(error) = runtime.block_on(news_flash.move_category(&category_id, &parent_id)) {
                            Util::send(&sender, Action::Error("Failed to move category.".to_owned(), error));
                        }
                    }
                    FeedListDndAction::MoveFeed(feed_id, from_id, to_id, _sort_index) => {
                        if let Err(error) = runtime.block_on(news_flash.move_feed(&feed_id, &from_id, &to_id)) {
                            Util::send(&sender, Action::Error("Failed to move feed.".to_owned(), error));
                        }
                    }
                }
            }
            Util::send(&sender, Action::UpdateSidebar);
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn export_article(&self) {
        let (sender, receiver) = oneshot::channel::<()>();

        if let Some(article) = self.window.content_page.read().article_view_visible_article() {
            let dialog = FileChooserDialog::with_buttons(
                Some("Export Article"),
                Some(&self.window.widget),
                FileChooserAction::Save,
                &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Ok)],
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

            if let ResponseType::Ok = dialog.run() {
                self.window.content_header.start_more_actions_spinner();

                let news_flash = self.news_flash.clone();
                let global_sender = self.sender.clone();
                let settings = self.settings.clone();
                let filename = match dialog.get_filename() {
                    Some(filename) => filename,
                    None => {
                        Util::send(&self.sender, Action::ErrorSimpleMessage("No filename set.".to_owned()));
                        return;
                    }
                };
                let thread_future = async move {
                    if let Some(news_flash) = news_flash.read().as_ref() {
                        let article = match Runtime::new()
                            .expect(RUNTIME_ERROR)
                            .block_on(news_flash.article_download_images(&article.article_id))
                        {
                            Ok(article) => article,
                            Err(error) => {
                                Util::send(
                                    &global_sender,
                                    Action::Error("Failed to downlaod article images.".to_owned(), error),
                                );
                                sender.send(()).unwrap();
                                return;
                            }
                        };

                        sender.send(()).unwrap();

                        let (feeds, _) = match news_flash.get_feeds() {
                            Ok(feeds) => feeds,
                            Err(error) => {
                                Util::send(
                                    &global_sender,
                                    Action::Error("Failed to load feeds from db.".to_owned(), error),
                                );
                                return;
                            }
                        };
                        let feed = match feeds.iter().find(|&f| f.feed_id == article.feed_id) {
                            Some(feed) => feed,
                            None => {
                                Util::send(
                                    &global_sender,
                                    Action::ErrorSimpleMessage("Failed to find specific feed.".to_owned()),
                                );
                                return;
                            }
                        };
                        let html =
                            ArticleView::build_article_static("article", &article, &feed.label, &settings, None, None);
                        if FileUtil::write_text_file(&filename, &html).is_err() {
                            Util::send(
                                &global_sender,
                                Action::ErrorSimpleMessage("Failed to write OPML data to disc.".to_owned()),
                            );
                        }
                    }
                };

                let content_header = self.window.content_header.clone();
                let glib_future = receiver.map(move |_res| {
                    content_header.stop_more_actions_spinner();
                });

                self.threadpool.spawn_ok(thread_future);
                Util::glib_spawn_future(glib_future);
            }
            dialog.emit_close();
        }
    }

    fn export_opml(&self) {
        let dialog = FileChooserDialog::with_buttons(
            Some("Export OPML"),
            Some(&self.window.widget),
            FileChooserAction::Save,
            &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Ok)],
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

        if let ResponseType::Ok = dialog.run() {
            if let Some(news_flash) = self.news_flash.read().as_ref() {
                let opml = match news_flash.export_opml() {
                    Ok(opml) => opml,
                    Err(error) => {
                        Util::send(
                            &self.sender,
                            Action::Error("Failed to get OPML data.".to_owned(), error),
                        );
                        return;
                    }
                };
                if let Some(filename) = dialog.get_filename() {
                    if FileUtil::write_text_file(&filename, &opml).is_err() {
                        Util::send(
                            &self.sender,
                            Action::ErrorSimpleMessage("Failed to write OPML data to disc.".to_owned()),
                        );
                    }
                }
            }
        }

        dialog.emit_close();
    }

    fn queue_quit(&self) {
        self.window.widget.close();
        self.window.execute_pending_undoable_action();
        
        // wait for ongoing sync to finish, but limit waiting to max 10s
        let sender = self.sender.clone();
        let news_flash = self.news_flash.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let wait_each_loop = time::Duration::from_millis(100);
                let mut total_waiting_time = time::Duration::from_millis(0);
                let max_wait_time = time::Duration::from_secs(10);

                while news_flash.is_sync_ongoing() && total_waiting_time < max_wait_time {
                    total_waiting_time += wait_each_loop;
                    thread::sleep(wait_each_loop);
                }
            }

            Util::send(&sender, Action::ForceQuit);
        };

        self.threadpool.spawn_ok(thread_future);
    }

    fn force_quit(&self) {
        self.application.quit();
    }
}
