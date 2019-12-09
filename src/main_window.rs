use crate::about_dialog::APP_NAME;
use crate::app::Action;
use crate::article_list::{MarkUpdate, ReadUpdate};
use crate::config::{APP_ID, PROFILE};
use crate::content_page::{ContentHeader, ContentPage};
use crate::error_bar::ErrorBar;
use crate::gtk_handle;
use crate::login_screen::{LoginHeaderbar, PasswordLogin, WebLogin};
use crate::main_window_actions::MainWindowActions;
use crate::main_window_state::MainWindowState;
use crate::responsive::ResponsiveLayout;
use crate::settings::{Keybindings, Settings};
use crate::sidebar::models::SidebarSelection;
use crate::undo_bar::{UndoActionModel, UndoBar};
use crate::util::{BuilderHelper, GtkHandle, GtkUtil, GTK_CSS_ERROR, GTK_RESOURCE_FILE_ERROR};
use crate::welcome_screen::{WelcomeHeaderbar, WelcomePage};
use crate::Resources;
use gdk::EventKey;
use glib::{self, Sender, Variant};
use gtk::{
    self, Application, ApplicationWindow, CssProvider, CssProviderExt, GtkWindowExt, Inhibit, Settings as GtkSettings,
    SettingsExt, Stack, StackExt, StackTransitionType, StyleContext, StyleContextExt, WidgetExt,
};
use log::{info, warn};
use news_flash::models::PluginID;
use news_flash::{NewsFlash, NewsFlashError};
use parking_lot::RwLock;
use std::cell::RefCell;
use std::rc::Rc;

const CONTENT_PAGE: &str = "content";

pub struct MainWindow {
    widget: ApplicationWindow,
    error_bar: ErrorBar,
    undo_bar: GtkHandle<UndoBar>,
    pub content_page: GtkHandle<ContentPage>,
    pub oauth_logn_page: WebLogin,
    pub password_login_page: PasswordLogin,
    stack: Stack,
    header_stack: Stack,
    state: GtkHandle<MainWindowState>,
    sender: Sender<Action>,
}

impl MainWindow {
    pub fn new(app: &Application, sender: Sender<Action>) -> Self {
        GtkUtil::register_symbolic_icons();
        let provider_handle = gtk_handle!(CssProvider::new());
        let settings = gtk_handle!(Settings::open().expect("Failed to access settings file"));

        if let Some(gtk_settings) = GtkSettings::get_default() {
            gtk_settings.set_property_gtk_application_prefer_dark_theme(settings.borrow().get_prefer_dark_theme());
        }

        // setup CSS for window
        Self::load_css(&provider_handle);

        let builder = BuilderHelper::new("main_window");
        let window = builder.get::<ApplicationWindow>("main_window");
        let stack = builder.get::<Stack>("main_stack");
        let header_stack = builder.get::<Stack>("header_stack");
        let undo_bar = UndoBar::new(&builder);
        let error_bar = ErrorBar::new(&builder);

        let responsive_layout = gtk_handle!(ResponsiveLayout::new(&builder));

        let _login_header = LoginHeaderbar::new(&builder, sender.clone());
        let _welcome_header = WelcomeHeaderbar::new(&builder);
        let content_header = ContentHeader::new(&builder);

        window.set_application(Some(app));
        window.set_icon_name(Some(APP_ID));
        window.set_title(APP_NAME);
        if PROFILE == "Devel" {
            window.get_style_context().add_class("devel");
        }

        let delete_event_settings = settings.clone();
        window.connect_delete_event(move |win, _| {
            if delete_event_settings.borrow().get_keep_running_in_background() {
                win.hide_on_delete();
            } else {
                GtkUtil::execute_action_main_window(win, "quit", None);
            }

            Inhibit(true)
        });

        // setup pages
        let _welcome = WelcomePage::new(&builder, sender.clone());
        let pw_login = PasswordLogin::new(&builder, sender.clone());
        let oauth_login = WebLogin::new(&builder, sender.clone());
        let content = ContentPage::new(&builder, &settings, sender.clone());

        let content_page_handle = gtk_handle!(content);
        let content_header_handle = gtk_handle!(content_header);
        let news_flash_handle = gtk_handle!(None);
        let undo_bar_handle = gtk_handle!(undo_bar);

        let state = gtk_handle!(MainWindowState::new());

        MainWindowActions::setup_schedule_sync_action(&window, &settings);
        MainWindowActions::setup_sync_action(&window, &sender, &content_header_handle, &news_flash_handle);
        MainWindowActions::setup_sidebar_selection_action(&window, &state, &responsive_layout);
        MainWindowActions::setup_update_sidebar_action(
            &window,
            &sender,
            &content_page_handle,
            &news_flash_handle,
            &state,
            &undo_bar_handle,
        );
        MainWindowActions::setup_headerbar_selection_action(&window, &content_header_handle, &state);
        MainWindowActions::setup_search_action(&window, &state);
        MainWindowActions::setup_update_article_list_action(
            &window,
            &sender,
            &state,
            &content_page_handle,
            &news_flash_handle,
            &undo_bar_handle,
        );
        MainWindowActions::setup_show_more_articles_action(
            &window,
            &sender,
            &state,
            &content_page_handle,
            &news_flash_handle,
            &undo_bar_handle,
        );
        MainWindowActions::setup_show_article_action(
            &window,
            &sender,
            &content_page_handle,
            &content_header_handle,
            &news_flash_handle,
            &responsive_layout,
        );
        MainWindowActions::setup_close_article_action(&window, &content_page_handle, &content_header_handle);
        MainWindowActions::setup_redraw_article_action(&window, &content_page_handle);
        MainWindowActions::setup_mark_article_read_action(
            &window,
            &sender,
            &news_flash_handle,
            &content_page_handle,
            &content_header_handle,
        );
        MainWindowActions::setup_mark_article_action(
            &window,
            &sender,
            &news_flash_handle,
            &content_page_handle,
            &content_header_handle,
        );
        MainWindowActions::setup_rename_feed_action(&window, &sender, &news_flash_handle);
        MainWindowActions::setup_add_action(&window, &sender, &news_flash_handle, &content_page_handle);
        MainWindowActions::setup_rename_category_action(&window, &sender, &news_flash_handle);
        MainWindowActions::setup_delete_selection_action(&window, &sender, &content_page_handle);
        MainWindowActions::setup_delete_feed_action(&window, &sender, &news_flash_handle);
        MainWindowActions::setup_delete_category_action(&window, &sender, &news_flash_handle);
        MainWindowActions::setup_move_action(&window, &sender, &news_flash_handle);
        MainWindowActions::setup_about_action(&window);
        MainWindowActions::setup_settings_action(&window, &sender, &settings);
        MainWindowActions::setup_shortcut_window_action(&window, &settings);
        MainWindowActions::setup_quit_action(&window, app);
        MainWindowActions::setup_export_action(&window, &sender, &news_flash_handle);
        MainWindowActions::setup_export_article_action(
            &window,
            &sender,
            &news_flash_handle,
            &content_page_handle,
            &settings,
        );
        MainWindowActions::setup_select_next_article_action(&window, &content_page_handle);
        MainWindowActions::setup_select_prev_article_action(&window, &content_page_handle);
        MainWindowActions::setup_sidebar_set_read_action(
            &window,
            &sender,
            &news_flash_handle,
            &state,
            &content_page_handle,
            &content_header_handle,
        );
        MainWindowActions::setup_toggle_article_read_action(&window, &content_page_handle);
        MainWindowActions::setup_toggle_article_marked_action(&window, &content_page_handle);

        Self::setup_shortcuts(
            &window,
            &sender,
            &content_page_handle,
            &stack,
            &settings,
            &content_header_handle,
        );

        if let Ok(news_flash_lib) = NewsFlash::try_load(&crate::app::DATA_DIR) {
            info!("Successful load from config");

            stack.set_visible_child_name(CONTENT_PAGE);
            header_stack.set_visible_child_name(CONTENT_PAGE);

            if let Some(id) = news_flash_lib.id() {
                let content_page_handle_clone = content_page_handle.clone();
                let sender = sender.clone();
                if content_page_handle_clone
                    .borrow()
                    .set_service(&id, news_flash_lib.user_name())
                    .is_err()
                {
                    GtkUtil::send(
                        &sender,
                        Action::ErrorSimpleMessage("Failed to set sidebar service logo.".to_owned()),
                    );
                }

                *news_flash_handle.borrow_mut() = Some(news_flash_lib);

                // try to fill content page with data
                if content_page_handle
                    .borrow_mut()
                    .update_sidebar(&news_flash_handle, &state, &undo_bar_handle)
                    .is_err()
                {
                    GtkUtil::send(
                        &sender,
                        Action::ErrorSimpleMessage("Failed to populate sidebar with data.".to_owned()),
                    );
                }
                if content_page_handle
                    .borrow_mut()
                    .update_article_list(&news_flash_handle, &state, &undo_bar_handle)
                    .is_err()
                {
                    GtkUtil::send(
                        &sender,
                        Action::ErrorSimpleMessage("Failed to populate article list with data.".to_owned()),
                    );
                }

                // schedule background sync
                GtkUtil::execute_action_main_window(&window, "schedule-sync", None);
            } else {
                warn!("No valid backend ID");
                stack.set_visible_child_name("welcome");
                header_stack.set_visible_child_name("welcome");
            }
        } else {
            warn!("No account configured");
            stack.set_visible_child_name("welcome");
            header_stack.set_visible_child_name("welcome");
        }

        if let Some(gtk_settings) = GtkSettings::get_default() {
            gtk_settings.set_property_gtk_application_prefer_dark_theme(settings.borrow().get_prefer_dark_theme());

            let window = window.clone();
            let provider = provider_handle.clone();
            gtk_settings.connect_property_gtk_application_prefer_dark_theme_notify(move |_settings| {
                Self::load_css(&provider);
                GtkUtil::execute_action_main_window(&window, "redraw-article", None);
            });
        }

        MainWindow {
            widget: window,
            error_bar,
            undo_bar: undo_bar_handle,
            content_page: content_page_handle,
            oauth_logn_page: oauth_login,
            password_login_page: pw_login,
            stack,
            header_stack,
            state,
            sender,
        }
    }

    pub fn widget(&self) -> ApplicationWindow {
        self.widget.clone()
    }

    fn setup_shortcuts(
        window: &ApplicationWindow,
        sender: &Sender<Action>,
        content_page: &GtkHandle<ContentPage>,
        main_stack: &Stack,
        settings: &GtkHandle<Settings>,
        content_header: &GtkHandle<ContentHeader>,
    ) {
        let main_stack = main_stack.clone();
        let sender = sender.clone();
        let settings = settings.clone();
        let content_page = content_page.clone();
        let main_window = window.clone();
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
                GtkUtil::execute_action_main_window(&widget, "shortcuts", None);
            }

            if Self::check_shortcut("refresh", &settings, event) {
                GtkUtil::execute_action_main_window(&widget, "sync", None);
            }

            if Self::check_shortcut("quit", &settings, event) {
                GtkUtil::execute_action_main_window(&widget, "quit", None);
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
                GtkUtil::execute_action_main_window(&widget, "next-article", None);
            }

            if Self::check_shortcut("previous_article", &settings, event) {
                GtkUtil::execute_action_main_window(&widget, "prev-article", None);
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

                        let update_data = serde_json::to_string(&update).expect("Failed to serialize ReadUpdate.");
                        let update_data = Variant::from(&update_data);
                        GtkUtil::execute_action_main_window(&main_window, "mark-article-read", Some(&update_data));
                        GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
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

                        let update_data = serde_json::to_string(&update).expect("Failed to serialize MarkUpdate.");
                        let update_data = Variant::from(&update_data);
                        GtkUtil::execute_action_main_window(&main_window, "mark-article", Some(&update_data));
                        GtkUtil::execute_action_main_window(&main_window, "update-article-list", None);
                    }
                }
            }

            if Self::check_shortcut("open_browser", &settings, event) {
                let article_model = content_page.borrow().get_selected_article_model();
                if let Some(article_model) = article_model {
                    if let Some(url) = article_model.url {
                        if gtk::show_uri_on_window(Some(&main_window), url.get().as_str(), 0).is_err() {
                            GtkUtil::send(
                                &sender,
                                Action::ErrorSimpleMessage("Failed to open URL in browser.".to_owned()),
                            );
                        }
                    } else {
                        warn!("Open selected article in browser: No url available.")
                    }
                } else {
                    warn!("Open selected article in browser: No article Selected.")
                }
            }

            if Self::check_shortcut("next_item", &settings, event)
                && content_page.borrow().sidebar_select_next_item().is_err()
            {
                GtkUtil::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select next item in sidebar.".to_owned()),
                );
            }

            if Self::check_shortcut("previous_item", &settings, event)
                && content_page.borrow().sidebar_select_prev_item().is_err()
            {
                GtkUtil::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select previous item in sidebar.".to_owned()),
                );
            }

            if Self::check_shortcut("scroll_up", &settings, event)
                && content_page.borrow().article_view_scroll_diff(-150.0).is_err()
            {
                GtkUtil::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select scroll article view up.".to_owned()),
                );
            }

            if Self::check_shortcut("scroll_down", &settings, event)
                && content_page.borrow().article_view_scroll_diff(150.0).is_err()
            {
                GtkUtil::send(
                    &sender,
                    Action::ErrorSimpleMessage("Failed to select scroll article view down.".to_owned()),
                );
            }

            if Self::check_shortcut("sidebar_set_read", &settings, event) {
                GtkUtil::execute_action_main_window(&main_window, "sidebar-set-read", None);
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
                        if Keybindings::clean_modifier(event.get_state()).is_empty() {
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

    pub fn show_error_simple_message(&self, msg: &str) {
        self.error_bar.simple_message(msg);
    }

    pub fn show_error(&self, msg: &str, error: NewsFlashError) {
        self.error_bar.news_flash_error(msg, error);
    }

    pub fn show_undo_bar(&self, action: UndoActionModel) {
        let select_all_button = match self.content_page.borrow().sidebar_get_selection() {
            SidebarSelection::All => false,
            SidebarSelection::Cateogry((selected_id, _label)) => match &action {
                UndoActionModel::DeleteCategory((delete_id, _label)) => &selected_id == delete_id,
                _ => false,
            },
            SidebarSelection::Feed((selected_id, _label)) => match &action {
                UndoActionModel::DeleteFeed((delete_id, _label)) => &selected_id == delete_id,
                _ => false,
            },
            SidebarSelection::Tag((selected_id, _label)) => match &action {
                UndoActionModel::DeleteTag((delete_id, _label)) => &selected_id == delete_id,
                _ => false,
            },
        };
        if select_all_button {
            self.state.borrow_mut().set_sidebar_selection(SidebarSelection::All);
            self.content_page.borrow().sidebar_select_all_button_no_update();
        }

        self.undo_bar.borrow().add_action(action);
    }

    pub fn show_welcome_page(&self) {
        self.header_stack.set_visible_child_name("welcome");
        self.password_login_page.reset();
        self.oauth_logn_page.reset();
        self.stack.set_transition_type(StackTransitionType::SlideRight);
        self.stack.set_visible_child_name("welcome");
    }

    pub fn show_password_login_page(&self, plugin_id: &PluginID) {
        if let Some(service_meta) = NewsFlash::list_backends().get(plugin_id) {
            if let Ok(()) = self.password_login_page.set_service(service_meta.clone()) {
                self.header_stack.set_visible_child_name("login");
                self.stack.set_transition_type(StackTransitionType::SlideLeft);
                self.stack.set_visible_child_name("password_login");
            }
        }
    }

    pub fn show_oauth_login_page(&self, plugin_id: &PluginID) {
        if let Some(service_meta) = NewsFlash::list_backends().get(plugin_id) {
            if let Ok(()) = self.oauth_logn_page.set_service(service_meta.clone()) {
                self.header_stack.set_visible_child_name("login");
                self.stack.set_transition_type(StackTransitionType::SlideLeft);
                self.stack.set_visible_child_name("oauth_login");
            }
        }
    }

    pub fn show_content_page(&self, plugin_id: &PluginID, news_flash: &RwLock<Option<NewsFlash>>) {
        if let Some(news_flash) = news_flash.read().as_ref() {
            let user_name: Option<String> = news_flash.user_name();
            self.stack.set_transition_type(StackTransitionType::SlideLeft);
            self.stack.set_visible_child_name("content");
            self.header_stack.set_visible_child_name("content");

            GtkUtil::execute_action_main_window(&self.widget, "update-sidebar", None);

            if self.content_page.borrow().set_service(&plugin_id, user_name).is_err() {
                GtkUtil::send(
                    &self.sender,
                    Action::ErrorSimpleMessage("Failed to set service.".to_owned()),
                );
            }
        }
    }
}
