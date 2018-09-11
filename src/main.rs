extern crate gio;
extern crate gtk;

use std::env::args;
use gio::prelude::*;
use gtk::prelude::*;

fn main() {
    let application = gtk::Application::new("com.gitlab.newsflash", gio::ApplicationFlags::empty())
        .expect("Initialization failed...");

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

        let button = gtk::Button::new_with_label("Click me!");

        window.add(&button);

        window.show_all();
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}
