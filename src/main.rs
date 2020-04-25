mod about_dialog;
mod add_dialog;
mod app;
mod article_list;
mod article_view;
mod color;
mod config;
mod content_page;
mod discover;
mod error_bar;
mod error_dialog;
mod i18n;
mod login_screen;
mod main_window;
mod main_window_state;
mod rename_dialog;
mod reset_page;
mod responsive;
mod settings;
mod sidebar;
mod tag_popover;
mod undo_bar;
mod util;
mod welcome_screen;

use crate::app::App;
use crate::config::APP_ID;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use rust_embed::RustEmbed;
use std::str;

#[derive(RustEmbed)]
#[folder = "data/resources/"]
struct Resources;

fn main() {
    // nicer backtrace
    color_backtrace::install();

    // Logging
    let encoder = PatternEncoder::new("{d(%H:%M:%S)} - {h({({l}):5.5})} - {m:<35.} (({M}:{L}))\n");
    let stdout = ConsoleAppender::builder().encoder(Box::new(encoder)).build();
    let appender = Appender::builder().build("stdout", Box::new(stdout));
    let root = Root::builder().appender("stdout").build(LevelFilter::Debug);
    let config = Config::builder()
        .appender(appender)
        .build(root)
        .expect("Failed to create log4rs config.");
    let _handle = log4rs::init_config(config).expect("Failed to init log4rs config.");

    // Gtk setup
    gtk::init().expect("Error initializing gtk.");
    glib::set_application_name("NewsFlash");
    glib::set_prgname(Some("NewsFlashGTK"));
    gtk::Window::set_default_icon_name(APP_ID);

    // Run app itself
    let app = App::new();
    app.run(app.clone());
}
