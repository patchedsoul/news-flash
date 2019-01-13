use gtk::{
    self,
    Application,
    GtkWindowExt,
    WidgetExt,
    Inhibit,
    CssProvider,
    CssProviderExt,
    GtkWindowExtManual,
    StyleContext,
    Builder,
    StackExt,
};
use crate::welcome_screen::{
    WelcomePage,
};
use crate::login_screen::{
    password_login::PasswordLogin,
};
use news_flash::models::{
    PluginMetadata,
};
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;


#[derive(Clone, Debug)]
pub struct MainWindow {
    widget: gtk::ApplicationWindow,
    stack: gtk::Stack,
}

impl MainWindow {
    pub fn new(app: &Application) -> Result<Self, Error> {

        // setup CSS for window
        let provider = CssProvider::new();
        let css_data = Resources::get("css/app.css").ok_or(format_err!("some err"))?;
        CssProvider::load_from_data(&provider, css_data.as_ref()).unwrap();
        StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().unwrap(),
            &provider,
            600,
        );

        let ui_data = Resources::get("ui/main_window.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let window : gtk::ApplicationWindow = builder.get_object("main_window").ok_or(format_err!("some err"))?;
        let stack : gtk::Stack = builder.get_object("main_stack").ok_or(format_err!("some err"))?;

        window.set_application(app);
        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        let welcome = Self::setup_welcome_page()?;
        stack.add_named(&welcome.widget(), "welcome");

        let pw_login = Self::setup_password_login_page()?;
        stack.add_named(&pw_login.widget(), "password_login");
        
        stack.set_visible_child_name("password_login");
        window.show_all();

        Ok(MainWindow {
            widget: window,
            stack: stack,
        })
    }

    fn setup_welcome_page() -> Result<WelcomePage, Error> {
        let welcome = WelcomePage::new()?;
        Ok(welcome)
    }

    fn setup_password_login_page() -> Result<PasswordLogin, Error> {
        let pw_login = PasswordLogin::new()?;
        Ok(pw_login)
    }

    pub fn present(&self) {
        self.widget.present();
    }
}