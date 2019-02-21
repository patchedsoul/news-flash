mod main_window;
mod main_window_actions;
mod sidebar;
mod welcome_screen;
mod login_screen;
mod gtk_util;
mod error_dialog;
mod content_page;

use gio::{
    self,
    ApplicationExt,
    ApplicationExtManual,
};
use gtk::{
    self,
    Application,
};
use crate::main_window::MainWindow;
use rust_embed::RustEmbed;
use std::env::args;
use std::str;
use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use log4rs::config::{
    Appender,
    Config,
    Root
};

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Resources;

fn main() {
    let application = Application::new("com.gitlab.newsflash", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    let encoder = PatternEncoder::new("{d(%H:%M:%S)} - {h({({l}):5.5})} - {m:<35.} (({M}:{L}))\n");

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(encoder))
        .build();

    let appender = Appender::builder()
        .build("stdout", Box::new(stdout));

    let root = Root::builder()
        .appender("stdout")
        .build(LevelFilter::Debug);

    let config = Config::builder()
        .appender(appender)
        .build(root)
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();

    application.connect_startup(move |_app| {

    });
    application.connect_activate(move |app| {
        let mainwindow = MainWindow::new(&app).unwrap();
        mainwindow.present();
    });

    application.run(&args().collect::<Vec<_>>());
}


