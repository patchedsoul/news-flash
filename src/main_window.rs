use crate::about_dialog::APP_NAME;
use crate::article_list::{MarkUpdate, ReadUpdate};
use crate::config::{APP_ID, PROFILE};
use crate::content_page::{ContentHeader, ContentPage};
use crate::gtk_handle;
use crate::login_screen::{LoginHeaderbar, PasswordLogin, WebLogin};
use crate::main_window_actions::MainWindowActions;
use crate::main_window_state::MainWindowState;
use crate::responsive::ResponsiveLayout;
use crate::settings::{Keybindings, Settings};
use crate::sidebar::models::SidebarSelection;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil, GTK_CSS_ERROR, GTK_RESOURCE_FILE_ERROR};
use crate::welcome_screen::{WelcomeHeaderbar, WelcomePage};
use crate::Resources;
use failure::Error;
use gdk::EventKey;
use gio::{ActionExt, ActionMapExt};
use glib::{self, Variant};
use gtk::{
    self, Application, ApplicationWindow, CssProvider, CssProviderExt, GtkWindowExt, GtkWindowExtManual, Inhibit, InfoBar,
    Settings as GtkSettings, SettingsExt, Stack, StackExt, StyleContext, StyleContextExt, WidgetExt,
};
use lazy_static::lazy_static;
use log::{info, warn};
use news_flash::NewsFlash;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

lazy_static! {
    pub static ref DATA_DIR: PathBuf = glib::get_user_config_dir()
        .expect("Failed to find the config dir")
        .join("news-flash");
}

const CONTENT_PAGE: &str = "content";

pub struct MainWindow {
    widget: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application) -> Result<Self, Error> {
        let provider_handle = gtk_handle!(CssProvider::new());

        let settings = gtk_handle!(Settings::open()?);

        if let Some(gtk_settings) = GtkSettings::get_default() {
            gtk_settings.set_property_gtk_application_prefer_dark_theme(settings.borrow().get_prefer_dark_theme());
        }

        // setup CSS for window
        Self::load_css(&provider_handle);

        let builder = BuilderHelper::new("main_window");
        let window = builder.get::<ApplicationWindow>("main_window");
        let stack = builder.get::<Stack>("main_stack");
        let header_stack = builder.get::<Stack>("header_stack");
        let undo_bar = builder.get::<InfoBar>("undo_bar");

        let responsive_layout = gtk_handle!(ResponsiveLayout::new(&builder));

        let _login_header = LoginHeaderbar::new(&builder);
        let _welcome_header = WelcomeHeaderbar::new(&builder);
        let content_header = ContentHeader::new(&builder);

        window.set_application(Some(app));
        window.set_icon_name(Some(APP_ID));
        window.set_title(APP_NAME);
        if PROFILE == "Devel" {
            window.get_style_context().add_class("devel");
        }
        window.connect_delete_event(move |win, _| {
            win.destroy();
            Inhibit(false)
        });

        // setup pages
        let _welcome = WelcomePage::new(&builder)?;
        let pw_login = PasswordLogin::new(&builder);
        let oauth_login = WebLogin::new(&builder);
        let content = ContentPage::new(&builder, &settings)?;

        let pw_login_handle = gtk_handle!(pw_login);
        let oauht_login_handle = gtk_handle!(oauth_login);
        let content_page_handle = gtk_handle!(content);
        let content_header_handle = gtk_handle!(content_header);
        let news_flash_handle = gtk_handle!(None);

        let state = gtk_handle!(MainWindowState::new());

        MainWindowActions::setup_show_password_page_action(&window, &pw_login_handle, &stack, &header_stack);
        MainWindowActions::setup_show_oauth_page_action(&window, &oauht_login_handle, &stack, &header_stack);
        MainWindowActions::setup_show_welcome_page_action(
            &window,
            &oauht_login_handle,
            &pw_login_handle,
            &stack,
            &header_stack,
        );
        MainWindowActions::setup_show_content_page_action(
            &window,
            &news_flash_handle,
            &stack,
            &header_stack,
            &content_page_handle,
            &state,
        );
        MainWindowActions::setup_login_action(&window, &news_flash_handle, &oauht_login_handle, &pw_login_handle);
        MainWindowActions::setup_sync_action(&window, &content_header_handle, &news_flash_handle);
        MainWindowActions::setup_sidebar_selection_action(&window, &state, &responsive_layout);
        MainWindowActions::setup_update_sidebar_action(&window, &content_page_handle, &news_flash_handle, &state);
        MainWindowActions::setup_headerbar_selection_action(&window, &content_header_handle, &state);
        MainWindowActions::setup_search_action(&window, &state);
        MainWindowActions::setup_update_article_list_action(&window, &state, &content_page_handle, &news_flash_handle);
        MainWindowActions::setup_show_more_articles_action(&window, &state, &content_page_handle, &news_flash_handle);
        MainWindowActions::setup_show_article_action(
            &window,
            &content_page_handle,
            &content_header_handle,
            &news_flash_handle,
            &responsive_layout,
        );
        MainWindowActions::setup_close_article_action(&window, &content_page_handle, &content_header_handle);
        MainWindowActions::setup_redraw_article_action(&window, &content_page_handle);
        MainWindowActions::setup_mark_article_read_action(&window, &news_flash_handle);
        MainWindowActions::setup_mark_article_action(&window, &news_flash_handle);
        MainWindowActions::setup_rename_feed_action(&window, &news_flash_handle);
        MainWindowActions::setup_delete_feed_action(&window, &news_flash_handle, &undo_bar);
        MainWindowActions::setup_about_action(&window);
        MainWindowActions::setup_settings_action(&window, &settings);
        MainWindowActions::setup_shortcut_window_action(&window, &settings);
        MainWindowActions::setup_quit_action(&window, app);
        MainWindowActions::setup_export_action(&window, &news_flash_handle);
        MainWindowActions::setup_select_next_article_action(&window, &content_page_handle);
        MainWindowActions::setup_select_prev_article_action(&window, &content_page_handle);

        Self::setup_shortcuts(
            &window,
            &content_page_handle,
            &stack,
            &settings,
            &news_flash_handle,
            &content_header_handle,
        );

        if let Ok(news_flash_lib) = NewsFlash::try_load(&DATA_DIR) {
            info!("Successful load from config");

            stack.set_visible_child_name(CONTENT_PAGE);
            let id = news_flash_lib.id().unwrap();
            content_page_handle
                .borrow()
                .set_service(&id, news_flash_lib.user_name())?;
            *news_flash_handle.borrow_mut() = Some(news_flash_lib);

            // try to fill content page with data
            content_page_handle
                .borrow_mut()
                .update_sidebar(&news_flash_handle, &state);
            content_page_handle
                .borrow_mut()
                .update_article_list(&news_flash_handle, &state);

            header_stack.set_visible_child_name(CONTENT_PAGE);
        } else {
            warn!("No account configured");
            stack.set_visible_child_name("welcome");
            header_stack.set_visible_child_name("welcome");
        }

        if let Some(settings) = GtkSettings::get_default() {
            let window = window.clone();
            let provider = provider_handle.clone();
            settings.connect_property_gtk_application_prefer_dark_theme_notify(move |_settings| {
                Self::load_css(&provider);
                if let Some(action) = window.lookup_action("redraw-article") {
                    action.activate(None);
                }
            });
        }

        window.show_all();

        let main_window = MainWindow { widget: window };

        Ok(main_window)
    }

    pub fn present(&self) {
        self.widget.present();
    }

    fn setup_shortcuts(
        window: &ApplicationWindow,
        content_page: &GtkHandle<ContentPage>,
        main_stack: &Stack,
        settings: &GtkHandle<Settings>,
        news_flash: &GtkHandle<Option<NewsFlash>>,
        content_header: &GtkHandle<ContentHeader>,
    ) {
        let main_stack = main_stack.clone();
        let settings = settings.clone();
        let content_page = content_page.clone();
        let main_window = window.clone();
        let news_flash = news_flash.clone();
        let content_header = content_header.clone();
        window.connect_key_press_event(move |widget, event| {
            // ignore shortcuts when not on content page
            if let Some(visible_child) = main_stack.get_visible_child_name() {
                if visible_child != CONTENT_PAGE {
                    return Inhibit(false);
                }
            }

            // ignore shortcuts when typing in search entry
            if content_header.borrow().is_search_focused() {
                return Inhibit(false);
            }

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
                content_header.borrow().focus_search();
            }

            if Self::check_shortcut("all_articles", &settings, event) {
                content_header.borrow().select_all_button();
            }

            if Self::check_shortcut("only_unread", &settings, event) {
                content_header.borrow().select_unread_button();
            }

            if Self::check_shortcut("only_starred", &settings, event) {
                content_header.borrow().select_marked_button();
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
                        gtk::show_uri_on_window(Some(&main_window), url.get().as_str(), 0).unwrap();
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

            if Self::check_shortcut("scroll_up", &settings, event) {
                content_page.borrow().article_view_scroll_diff(-150.0).unwrap();
            }

            if Self::check_shortcut("scroll_down", &settings, event) {
                content_page.borrow().article_view_scroll_diff(150.0).unwrap();
            }

            if Self::check_shortcut("sidebar_set_read", &settings, event) {
                if let Some(news_flash) = news_flash.borrow_mut().as_mut() {
                    match content_page.borrow().sidebar_get_selection() {
                        SidebarSelection::All => news_flash.set_all_read().unwrap(),
                        SidebarSelection::Cateogry((category_id, _title)) => {
                            news_flash.set_category_read(&vec![category_id]).unwrap()
                        }
                        SidebarSelection::Feed((feed_id, _title)) => news_flash.set_feed_read(&vec![feed_id]).unwrap(),
                        SidebarSelection::Tag((tag_id, _title)) => news_flash.set_tag_read(&vec![tag_id]).unwrap(),
                    }
                }

                if let Some(action) = main_window.lookup_action("update-article-list") {
                    action.activate(None);
                }
                if let Some(action) = main_window.lookup_action("update-sidebar") {
                    action.activate(None);
                }
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
                            return true;
                        }
                    } else if event.get_state().contains(modifier) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn load_css(provider: &GtkHandle<CssProvider>) {
        let screen = gdk::Screen::get_default().expect(GTK_CSS_ERROR);

        // remove old style provider
        StyleContext::remove_provider_for_screen(&screen, &*provider.borrow());

        // setup new style provider
        let style_sheet = if let Some(settings) = GtkSettings::get_default() {
            if settings.get_property_gtk_application_prefer_dark_theme() {
                "app_dark"
            } else {
                "app"
            }
        } else {
            "app"
        };
        let css_data = Resources::get(&format!("css/{}.css", style_sheet)).expect(GTK_RESOURCE_FILE_ERROR);
        *provider.borrow_mut() = CssProvider::new();
        CssProvider::load_from_data(&*provider.borrow(), css_data.as_ref()).expect(GTK_CSS_ERROR);

        // apply new style provider
        StyleContext::add_provider_for_screen(&screen, &*provider.borrow(), 600);
    }
}
