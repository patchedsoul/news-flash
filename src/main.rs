mod main_window;
//mod sidebar;
mod welcome_screen;
mod login_screen;
mod gtk_util;

use cairo;
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
use failure;
use std::env::args;
use std::str;

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Resources;

fn main() {
    let application = Application::new("com.gitlab.newsflash", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    application.connect_startup(move |_app| {

    });
    application.connect_activate(move |app| {
        let mainwindow = MainWindow::new(&app).unwrap();
        mainwindow.present();
    });

    application.run(&args().collect::<Vec<_>>());
}


