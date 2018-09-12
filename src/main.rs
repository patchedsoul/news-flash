extern crate gio;
extern crate gtk;
extern crate gdk;
#[macro_use]
extern crate rust_embed;
extern crate news_flash;

mod sidebar;

use std::env::args;
use std::str;
use gio::prelude::*;
use gtk::prelude::*;
use sidebar::feed_list::category::Category;
use news_flash::models::{
    CategoryID,
};

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Resources;

fn main() {
    let application = gtk::Application::new("com.gitlab.newsflash", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

    let provider = gtk::CssProvider::new();
    let css_data = Resources::get("css/app.css").unwrap();
    gtk::CssProvider::load_from_data(&provider, &css_data).unwrap();
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().unwrap(),
        &provider,
        600,
    );

    application.connect_startup(move |app| {
        let window = gtk::ApplicationWindow::new(app);

        window.set_title("First GTK+ Program");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        let category = Category::new(CategoryID::new("test123"), "test123");

        window.add(&category.widget);

        window.show_all();
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
