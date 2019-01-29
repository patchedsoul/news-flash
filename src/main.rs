mod main_window;
//mod sidebar;
mod welcome_screen;
mod login_screen;
mod gtk_util;
mod error_dialog;

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
use log::{
    warn,
    info,
    error,
    trace,
};

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Resources;

fn main() {
    let application = Application::new("com.gitlab.newsflash", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} - {({l}):5.5} - {f}:{L} - {m}\n")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();

    info!("some info");
    warn!("some warning");
    error!("some error");
    trace!("some trace");

    application.connect_startup(move |_app| {

    });
    application.connect_activate(move |app| {
        let mainwindow = MainWindow::new(&app).unwrap();
        mainwindow.present();
    });

    application.run(&args().collect::<Vec<_>>());
}


