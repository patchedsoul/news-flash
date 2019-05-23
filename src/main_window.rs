use crate::content_page::{ContentHeader, ContentPage};
use crate::gtk_handle;
use crate::login_screen::{LoginHeaderbar, PasswordLogin, WebLogin};
use crate::main_window_actions::MainWindowActions;
use crate::main_window_state::MainWindowState;
use crate::util::{BuilderHelper, GtkUtil, GTK_CSS_ERROR, GTK_RESOURCE_FILE_ERROR, GtkHandle};
use crate::welcome_screen::{WelcomeHeaderbar, WelcomePage};
use crate::Resources;
use crate::settings::{Settings, Keybindings};
use crate::about_dialog::{ICON_NAME, APP_NAME};
use crate::article_list::{ReadUpdate, MarkUpdate};
use failure::Error;
use gtk::{
    self, Application, ApplicationWindow, CssProvider, CssProviderExt, GtkWindowExt, GtkWindowExtManual, Inhibit,
    Stack, StackExt, StyleContext, WidgetExt,
};
use glib::Variant;
use gdk::EventKey;
use gio::{ActionExt, ActionMapExt};
use log::{info, warn};
use news_flash::NewsFlash;
use std::cell::RefCell;
use std::rc::Rc;
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub static ref DATA_DIR: PathBuf = dirs::home_dir().expect("$HOME not available").join(".news-flash");
}

const PANED_DEFAULT_POS: i32 = 600;
const CONTENT_PAGE: &str = "content";

pub struct MainWindow {
    widget: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application) -> Result<Self, Error> {
        // setup CSS for window
        let css_data = Resources::get("css/app.css").expect(GTK_RESOURCE_FILE_ERROR);
        let screen = gdk::Screen::get_default().expect(GTK_CSS_ERROR);
        let provider = CssProvider::new();
        CssProvider::load_from_data(&provider, css_data.as_ref()).expect(GTK_CSS_ERROR);
        StyleContext::add_provider_for_screen(&screen, &provider, 600);

        let builder = BuilderHelper::new("main_window");
        let window = builder.get::<ApplicationWindow>("main_window");
        let stack = builder.get::<Stack>("main_stack");

        let settings = gtk_handle!(Settings::open()?);

        let login_header = LoginHeaderbar::new(&window);
        let welcome_header = WelcomeHeaderbar::new()?;
        let content_header = ContentHeader::new()?;

        window.set_application(app);
        window.set_icon_name(ICON_NAME);
        window.set_title(APP_NAME);
        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        // setup pages
        let welcome = WelcomePage::new(&window)?;
        stack.add_named(&welcome.widget(), "welcome");

        let pw_login = PasswordLogin::new()?;
        stack.add_named(&pw_login.widget(), "password_login");

        let oauth_login = WebLogin::new();
        stack.add_named(&oauth_login.widget(), "oauth_login");

        let content = ContentPage::new(&settings)?;
        stack.add_named(&content.widget(), CONTENT_PAGE);

        let pw_login_handle = gtk_handle!(pw_login);
        let oauht_login_handle = gtk_handle!(oauth_login);
        let content_page_handle = gtk_handle!(content);
        let content_header_handle = gtk_handle!(content_header);
        let news_flash_handle = gtk_handle!(None);

        let state = gtk_handle!(MainWindowState::new());

        MainWindowActions::setup_show_password_page_action(&window, &pw_login_handle, &stack, login_header.widget());
        MainWindowActions::setup_show_oauth_page_action(&window, &oauht_login_handle, &stack, login_header.widget());
        MainWindowActions::setup_show_welcome_page_action(
            &window,
            &oauht_login_handle,
            &pw_login_handle,
            &stack,
            welcome_header.widget(),
        );
        MainWindowActions::setup_show_content_page_action(
            &window,
            &news_flash_handle,
            &stack,
            &content_page_handle,
            content_header_handle.borrow().widget(),
            &state,
        );
        MainWindowActions::setup_login_action(&window, &news_flash_handle, &oauht_login_handle, &pw_login_handle);
        MainWindowActions::setup_sync_paned_action(&window, &content_page_handle, &content_header_handle);
        MainWindowActions::setup_sync_action(&window, &content_header_handle, &news_flash_handle);
        MainWindowActions::setup_sidebar_selection_action(&window, &state);
        MainWindowActions::setup_update_sidebar_action(&window, &content_page_handle, &news_flash_handle, &state);
        MainWindowActions::setup_headerbar_selection_action(&window, &state);
        MainWindowActions::setup_search_action(&window, &state);
        MainWindowActions::setup_update_article_list_action(&window, &state, &content_page_handle, &news_flash_handle);
        MainWindowActions::setup_show_more_articles_action(&window, &state, &content_page_handle, &news_flash_handle);
        MainWindowActions::setup_show_article_action(&window, &content_page_handle, &news_flash_handle);
        MainWindowActions::setup_redraw_article_action(&window, &content_page_handle);
        MainWindowActions::setup_mark_article_read_action(&window, &news_flash_handle);
        MainWindowActions::setup_mark_article_action(&window, &news_flash_handle);
        MainWindowActions::setup_about_action(&window);
        MainWindowActions::setup_settings_action(&window, &settings);
        MainWindowActions::setup_shortcut_window_action(&window, &settings);
        MainWindowActions::setup_quit_action(&window, app);
        MainWindowActions::setup_focus_search_action(&window, &content_header_handle);
        MainWindowActions::setup_export_action(&window, &news_flash_handle);
        MainWindowActions::setup_select_next_article_action(&window, &content_page_handle);
        MainWindowActions::setup_select_prev_article_action(&window, &content_page_handle);

        Self::setup_shortcuts(&window, &content_page_handle, &stack, &settings);

        if let Ok(news_flash_lib) = NewsFlash::try_load(&DATA_DIR) {
            info!("Successful load from config");

            stack.set_visible_child_name(CONTENT_PAGE);
            let id = news_flash_lib.id().unwrap();
            content_page_handle
                .borrow()
                .set_service(&id, news_flash_lib.user_name())?;
            *news_flash_handle.borrow_mut() = Some(news_flash_lib);

            // try to fill content page with data
            content_page_handle.borrow_mut().update_sidebar(&news_flash_handle, &state);
            content_page_handle
                .borrow_mut()
                .update_article_list(&news_flash_handle, &state);

            window.set_titlebar(&content_header_handle.borrow().widget());
        } else {
            warn!("No account configured");
            stack.set_visible_child_name("welcome");
            window.set_titlebar(&welcome_header.widget());
        }

        content_header_handle.borrow().set_paned(PANED_DEFAULT_POS);
        content_page_handle.borrow().set_paned(PANED_DEFAULT_POS);
        window.show_all();

        let main_window = MainWindow { widget: window };

        Ok(main_window)
    }

    pub fn present(&self) {
        self.widget.present();
    }

    fn setup_shortcuts(window: &ApplicationWindow, content_page: &GtkHandle<ContentPage>, main_stack: &Stack, settings: &GtkHandle<Settings>) {
        let main_stack = main_stack.clone();
        let settings = settings.clone();
        let content_page = content_page.clone();
        let main_window = window.clone();
        window.connect_key_press_event(move |widget, event| {

            // ignore shortcuts when not on content page
            if let Some(visible_child) = main_stack.get_visible_child_name() {
                if visible_child != CONTENT_PAGE {
                    return Inhibit(false)
                }
            }

            // ignore shortcuts when focusing search box (or any other entry)
            // FIXME

            if Self::check_shortcut("shortcuts", &settings, event) {
                if let Some(action) = widget.lookup_action("shortcuts") {
                    action.activate(None);
                }
            }

            if Self::check_shortcut("refresh", &settings, event) {
                if let Some(action) = widget.lookup_action("sync") {
                    action.activate(None);
                }
            }

            if Self::check_shortcut("quit", &settings, event) {
                if let Some(action) = widget.lookup_action("quit") {
                    action.activate(None);
                }
            }

            if Self::check_shortcut("search", &settings, event) {
                if let Some(action) = widget.lookup_action("focus-search") {
                    action.activate(None);
                }
            }

            if Self::check_shortcut("next_article", &settings, event) {
                if let Some(action) = widget.lookup_action("next-article") {
                    action.activate(None);
                }
            }

            if Self::check_shortcut("previous_article", &settings, event) {
                if let Some(action) = widget.lookup_action("prev-article") {
                    action.activate(None);
                }
            }

            if Self::check_shortcut("toggle_category_expanded", &settings, event) {
                content_page.borrow().sidebar_expand_collase_category();
            }

            if Self::check_shortcut("toggle_read", &settings, event) {
                let article_model = content_page.borrow().get_selected_article_model();
                if let Some(article_model) = article_model {
                    if let Ok(main_window) = GtkUtil::get_main_window(&main_stack) {
                        let update = ReadUpdate {
                            article_id: article_model.id.clone(),
                            read: article_model.read.invert(),
                        };

                        let update_data = serde_json::to_string(&update).unwrap();
                        let update_data = Variant::from(&update_data);
                        if let Some(action) = main_window.lookup_action("mark-article-read") {
                            action.activate(Some(&update_data));
                        }
                        if let Some(action) = main_window.lookup_action("update-article-list") {
                            action.activate(None);
                        }
                    }
                }
            }

            if Self::check_shortcut("toggle_marked", &settings, event) {
                let article_model = content_page.borrow().get_selected_article_model();
                if let Some(article_model) = article_model {
                    if let Ok(main_window) = GtkUtil::get_main_window(&main_stack) {
                        let update = MarkUpdate {
                            article_id: article_model.id.clone(),
                            marked: article_model.marked.invert(),
                        };

                        let update_data = serde_json::to_string(&update).unwrap();
                        let update_data = Variant::from(&update_data);
                        if let Some(action) = main_window.lookup_action("mark-article") {
                            action.activate(Some(&update_data));
                        }
                        if let Some(action) = main_window.lookup_action("update-article-list") {
                            action.activate(None);
                        }
                    }
                }
            }

            if Self::check_shortcut("open_browser", &settings, event) {
                let article_model = content_page.borrow().get_selected_article_model();
                if let Some(article_model) = article_model {
                    if let Some(url) = article_model.url {
                        gtk::show_uri_on_window(&main_window, url.get().as_str(), 0).unwrap();
                    } else {
                        warn!("Open selected article in browser: No url available.")
                    }
                } else {
                    warn!("Open selected article in browser: No article Selected.")
                }
            }

            if Self::check_shortcut("next_item", &settings, event) {
                content_page.borrow().sidebar_select_next_item().unwrap();
            }

            if Self::check_shortcut("previous_item", &settings, event) {
                content_page.borrow().sidebar_select_prev_item().unwrap();
            }

            Inhibit(false)
        });
    }

    fn check_shortcut(id: &str, settings: &GtkHandle<Settings>, event: &EventKey) -> bool {
        if let Ok(keybinding) = Keybindings::read_keybinding(id, settings) {
            if let Some(keybinding) = keybinding {
                let (keyval, modifier) = gtk::accelerator_parse(&keybinding);

                if gdk::keyval_to_lower(keyval) == gdk::keyval_to_lower(event.get_keyval()) {
                    if modifier.is_empty() {
                        if Keybindings::clean_modifier(&event.get_state()).is_empty() {
                            return true
                        }
                    } else if event.get_state().contains(modifier) {
                        return true
                    }
                }
            }
        }
        false
    }
}
