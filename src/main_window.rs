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
    StackTransitionType,
    HeaderBar,
};
use crate::welcome_screen::{
    WelcomePage,
    WelcomeHeaderbar,
};
use crate::login_screen::{
    PasswordLogin,
    LoginHeaderbar,
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
    login_headerbar: HeaderBar,
    welcome_headerbar: HeaderBar,
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

        let login_header = LoginHeaderbar::new(&window)?;
        let welcome_header = WelcomeHeaderbar::new()?;

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
        window.set_titlebar(&welcome_header.widget());
        window.show_all();
        
        Self::setup_show_password_page_action(&window, pw_login, &stack, login_header.widget());
        Self::setup_show_welcome_page_action(&window, &stack, welcome_header.widget());

        Ok(MainWindow {
            widget: window,
            stack: stack,
            login_headerbar: login_header.widget(),
            welcome_headerbar: welcome_header.widget(),
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

    fn setup_show_password_page_action(window: &ApplicationWindow, pw_page: PasswordLogin, stack: &Stack, headerbar: HeaderBar) {
        let application_window = window.clone();
        let stack = stack.clone();
        let show_pw_page = SimpleAction::new("show-pw-page", glib::VariantTy::new("s").ok());
        show_pw_page.connect_activate(move |_action, data| {
            if let Some(data) = data {
                if let Some(id_string) = data.get_str() {
                    let id = PluginID::new(id_string);
                    if let Some(service_meta) = NewsFlash::list_backends().get(&id) {
                        if let Ok(gui_desc) = service_meta.login_gui() {
                            if let Ok(()) = pw_page.set_service(service_meta.metadata(), gui_desc) {
                                application_window.set_titlebar(&headerbar);
                                stack.set_transition_type(StackTransitionType::SlideLeft);
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

    fn setup_show_welcome_page_action(window: &ApplicationWindow, stack: &Stack, headerbar: HeaderBar) {
        let application_window = window.clone();
        let stack = stack.clone();
        let show_welcome_page = SimpleAction::new("show-welcome-page", None);
        show_welcome_page.connect_activate(move |_action, _data| {
            application_window.set_titlebar(&headerbar);
            stack.set_transition_type(StackTransitionType::SlideRight);
            stack.set_visible_child_name("welcome");
        });
        show_welcome_page.set_enabled(true);
        window.add_action(&show_welcome_page);
    }

    pub fn present(&self) {
        self.widget.present();
    }
}