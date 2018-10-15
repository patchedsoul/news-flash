extern crate gio;
extern crate gtk;
extern crate gdk;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate rust_embed;
extern crate news_flash;
extern crate chrono;
extern crate cairo;

mod sidebar;

use std::env::args;
use std::str;
use gio::prelude::*;
use gtk::prelude::*;
use sidebar::{
    FeedList,
    feed_list::models::{
        FeedListTree,
    },
};
use news_flash::models::{
    Category as CategoryModel,
    Feed as FeedModel,
    FeedMapping,
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
    gtk::CssProvider::load_from_data(&provider, css_data.as_ref()).unwrap();
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().unwrap(),
        &provider,
        600,
    );

    application.connect_startup(move |app| {
        let window = gtk::ApplicationWindow::new(app);

        window.set_title("FeedList test");
        window.set_border_width(0);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        let category_1 = CategoryModel {
            category_id: CategoryID::new("category_1"),
            label: "category 1".to_owned(),
            sort_index: None,
            parent: NEWSFLASH_TOPLEVEL.clone(),
        };
        let feed_1 = FeedModel {
            feed_id: FeedID::new("feed_1"),
            label: "Feed 1".to_owned(),
            website: None,
            feed_url: None,
            icon_url: None,
            sort_index: Some(2),
        };
        let mapping_1 = FeedMapping {
            feed_id: FeedID::new("feed_1"),
            category_id: CategoryID::new("category_1"),
        };
        let feed_2 = FeedModel {
            feed_id: FeedID::new("feed_2"),
            label: "Feed 2".to_owned(),
            website: None,
            feed_url: None,
            icon_url: None,
            sort_index: Some(1),
        };
        let mapping_2 = FeedMapping {
            feed_id: FeedID::new("feed_2"),
            category_id: CategoryID::new("category_1"),
        };
        let category_2 = CategoryModel {
            category_id: CategoryID::new("category_2"),
            label: "category 2".to_owned(),
            sort_index: Some(0),
            parent: CategoryID::new("category_1"),
        };
        let feed_3 = FeedModel {
            feed_id: FeedID::new("feed_3"),
            label: "Feed 3".to_owned(),
            website: None,
            feed_url: None,
            icon_url: None,
            sort_index: Some(0),
        };
        let mapping_3 = FeedMapping {
            feed_id: FeedID::new("feed_3"),
            category_id: CategoryID::new("category_2"),
        };

        
        let mut tree = FeedListTree::new();
        tree.add_category(&category_1, 7);
        tree.add_category(&category_2, 5);
        tree.add_feed(&feed_1, &mapping_1, 2);
        tree.add_feed(&feed_2, &mapping_2, 0);
        tree.add_feed(&feed_3, &mapping_3, 5);
        
        let mut list = FeedList::new().unwrap();
        list.update(tree);
        window.add(&list.widget);

        window.show_all();
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
