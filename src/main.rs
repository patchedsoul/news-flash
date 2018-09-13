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
use sidebar::{
    Category,
    Feed,
};
use news_flash::models::{
    Category as CategoryModel,
    Feed as FeedModel,
    CategoryID,
    FeedID,
    NEWSFLASH_TOPLEVEL,
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

        let category = CategoryModel {
            category_id: CategoryID::new("test123"),
            label: "test123".to_owned(),
            sort_index: None,
            parent: NEWSFLASH_TOPLEVEL.clone(),
        };
        let feed_1 = FeedModel {
            feed_id: FeedID::new("feed_1"),
            label: "Feed 1".to_owned(),
            website: None,
            feed_url: None,
            icon_url: None,
            order_id: None,
        };
        let feed_1 = Feed::new(&feed_1, &category.category_id);

        let category = Category::new(&category);
        category.add_feed(&feed_1);

        

        window.add(&category.widget);

        window.show_all();
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
