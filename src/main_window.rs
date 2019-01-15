use gtk::{
    self,
    Application,
    ApplicationWindow,
    GtkWindowExt,
    WidgetExt,
    Inhibit,
    CssProvider,
    CssProviderExt,
    GtkWindowExtManual,
    StyleContext,
    Builder,
    Stack,
    StackExt,
};
use crate::welcome_screen::{
    WelcomePage,
};
use crate::login_screen::{
    password_login::PasswordLogin,
};
use gio::{
    SimpleAction,
    SimpleActionExt,
    ActionMapExt,
};
use news_flash::models::{
    PluginID,
};
use news_flash::NewsFlash;
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;



#[derive(Clone, Debug)]
pub struct MainWindow {
    widget: ApplicationWindow,
    stack: Stack,
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
        let window : ApplicationWindow = builder.get_object("main_window").ok_or(format_err!("some err"))?;
        let stack : Stack = builder.get_object("main_stack").ok_or(format_err!("some err"))?;

        window.set_application(app);
        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        let welcome = Self::setup_welcome_page(&window)?;
        stack.add_named(&welcome.widget(), "welcome");

        let pw_login = Self::setup_password_login_page()?;
        stack.add_named(&pw_login.widget(), "password_login");
        
        stack.set_visible_child_name("welcome");
        window.show_all();
        
        Self::setup_show_password_page_action(&window, pw_login, stack.clone());

        Ok(MainWindow {
            widget: window,
            stack: stack,
        })
    }

    fn setup_welcome_page(window: &ApplicationWindow) -> Result<WelcomePage, Error> {
        let welcome = WelcomePage::new(window)?;
        Ok(welcome)
    }

    fn setup_password_login_page() -> Result<PasswordLogin, Error> {
        let pw_login = PasswordLogin::new()?;
        Ok(pw_login)
    }

    fn setup_show_password_page_action(window: &ApplicationWindow, pw_page: PasswordLogin, stack: Stack) {
        let show_pw_page = SimpleAction::new("show-pw-page", glib::VariantTy::new("s").ok());
        show_pw_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    if let Some(service_meta) = NewsFlash::list_backends().get(&id) {
                        if let Ok(gui_desc) = service_meta.login_gui() {
                            if let Ok(()) = pw_page.set_service(service_meta.metadata(), gui_desc) {
                                stack.set_visible_child_name("password_login");
                            }
                        }
                    }
                }
            }
        });
        show_pw_page.set_enabled(true);
        window.add_action(&show_pw_page);
    }

    pub fn present(&self) {
        self.widget.present();
    }
}