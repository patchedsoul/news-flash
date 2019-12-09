use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;

use gio::{ApplicationExt, ApplicationExtManual, Notification, NotificationPriority, ThemedIcon};
use glib::{translate::ToGlib, Receiver, Sender};
use gtk::{Application, Continue, GtkApplicationExt, GtkWindowExtManual, WidgetExt};
use lazy_static::lazy_static;
use log::error;
use news_flash::models::{LoginData, PluginID};
use news_flash::{NewsFlash, NewsFlashError};
use parking_lot::RwLock;

use crate::config::APP_ID;
use crate::main_window::MainWindow;
use crate::settings::Settings;
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
    Login(LoginData),
    ScheduleSync,
}
pub struct App {
    application: gtk::Application,
    window: MainWindow,
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,
    news_flash: RwLock<Option<NewsFlash>>,
    settings: Settings,
    sync_source_id: RwLock<Option<u32>>,
}

impl App {
    pub fn new() -> Rc<Self> {
        let application =
            Application::new(Some(APP_ID), gio::ApplicationFlags::empty()).expect("Initialization gtk-app failed");

        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let settings = Settings::open().expect("Failed to access settings file");
        let window = MainWindow::new(&application, &settings, sender.clone());

        let app = Rc::new(Self {
            application,
            window,
            sender,
            receiver,
            news_flash: RwLock::new(None),
            settings,
            sync_source_id: RwLock::new(None),
        });
        app.setup_signals();

        app
    }

    fn setup_signals(&self) {
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
            Action::ShowWelcomePage => self.window.show_welcome_page(),
            Action::ShowContentPage(plugin_id) => self.window.show_content_page(&plugin_id, &self.news_flash),
            Action::ShowPasswordLogin(plugin_id) => self.window.show_password_login_page(&plugin_id),
            Action::ShowOauthLogin(plugin_id) => self.window.show_oauth_login_page(&plugin_id),
            Action::Login(data) => self.login(data),
            Action::ScheduleSync => self.schedule_sync(),
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
        let sync_interval = self.settings.get_sync_interval();
        if let Some(sync_interval) = sync_interval.to_seconds() {
            let main_window = self.window.widget.clone();
            self.sync_source_id.write().replace(
                gtk::timeout_add_seconds(sync_interval, move || {
                    GtkUtil::execute_action_main_window(&main_window, "sync", None);
                    Continue(true)
                })
                .to_glib(),
            );
        } else {
            self.sync_source_id.write().take();
        }
    }
}
