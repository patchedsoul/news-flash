use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use gio::{ApplicationExt, ApplicationExtManual, Notification, NotificationPriority, ThemedIcon};
use glib::Receiver;
use gtk::{Application, GtkApplicationExt, GtkWindowExtManual, WidgetExt};
use news_flash::NewsFlashError;

use crate::config::APP_ID;
use crate::main_window::MainWindow;
use crate::undo_bar::UndoActionModel;

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
}
pub struct App {
    application: gtk::Application,
    window: MainWindow,
    receiver: RefCell<Option<Receiver<Action>>>,
}

impl App {
    pub fn new() -> Rc<Self> {
        let application =
            Application::new(Some(APP_ID), gio::ApplicationFlags::empty()).expect("Initialization gtk-app failed");

        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let window = MainWindow::new(&application, sender.clone());

        let app = Rc::new(Self {
            application,
            window,
            receiver,
        });
        app.setup_signals();

        app
    }

    fn setup_signals(&self) {
        let window = self.window.widget();
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
}
