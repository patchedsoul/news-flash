use crate::error_dialog::ErrorDialog;
use crate::util::GtkUtil;
use crate::Resources;
use failure::{format_err, Error, Fail};
use gio::{ActionExt, ActionMapExt};
use glib::{signal::SignalHandlerId, translate::ToGlib, Variant};
use gtk::{self, ButtonExt, EntryExt, ImageExt, InfoBarExt, LabelExt, ResponseType, RevealerExt, StyleContextExt, WidgetExt};
use news_flash::models::{LoginData, LoginGUI, PasswordLogin as PasswordLoginData, PasswordLoginGUI, PluginIcon, PluginInfo};
use news_flash::{FeedApiError, FeedApiErrorKind, NewsFlashError, NewsFlashErrorKind};
use std::str;

#[derive(Clone, Debug)]
pub struct PasswordLogin {
    page: gtk::Box,
    logo: gtk::Image,
    headline: gtk::Label,
    scale_factor: i32,
    url_label: gtk::Label,
    url_entry: gtk::Entry,
    user_entry: gtk::Entry,
    pass_entry: gtk::Entry,
    http_user_entry: gtk::Entry,
    http_pass_entry: gtk::Entry,
    http_revealer: gtk::Revealer,
    info_bar: gtk::InfoBar,
    info_bar_label: gtk::Label,
    login_button: gtk::Button,
    ignore_tls_button: gtk::Button,
    error_details_button: gtk::Button,
    info_bar_close_signal: Option<u64>,
    info_bar_response_signal: Option<u64>,
    url_entry_signal: Option<u64>,
    user_entry_signal: Option<u64>,
    pass_entry_signal: Option<u64>,
    http_user_entry_signal: Option<u64>,
    http_pass_entry_signal: Option<u64>,
    login_button_signal: Option<u64>,
    ignore_tls_signal: Option<u64>,
    error_details_signal: Option<u64>,
}

impl PasswordLogin {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/password_login.ui").ok_or_else(|| format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let page: gtk::Box = builder.get_object("password_login").ok_or_else(|| format_err!("some err"))?;
        let logo: gtk::Image = builder.get_object("logo").ok_or_else(|| format_err!("some err"))?;
        let headline: gtk::Label = builder.get_object("headline").ok_or_else(|| format_err!("some err"))?;
        let url_label: gtk::Label = builder.get_object("url_label").ok_or_else(|| format_err!("some err"))?;
        let url_entry: gtk::Entry = builder.get_object("url_entry").ok_or_else(|| format_err!("some err"))?;
        let user_entry: gtk::Entry = builder.get_object("user_entry").ok_or_else(|| format_err!("some err"))?;
        let pass_entry: gtk::Entry = builder.get_object("pass_entry").ok_or_else(|| format_err!("some err"))?;
        let http_user_entry: gtk::Entry = builder.get_object("http_user_entry").ok_or_else(|| format_err!("some err"))?;
        let http_pass_entry: gtk::Entry = builder.get_object("http_pass_entry").ok_or_else(|| format_err!("some err"))?;
        let http_revealer: gtk::Revealer = builder.get_object("http_auth_revealer").ok_or_else(|| format_err!("some err"))?;
        let login_button: gtk::Button = builder.get_object("login_button").ok_or_else(|| format_err!("some err"))?;
        let info_bar: gtk::InfoBar = builder.get_object("info_bar").ok_or_else(|| format_err!("some err"))?;
        let info_bar_label: gtk::Label = builder.get_object("info_bar_label").ok_or_else(|| format_err!("some err"))?;
        let ignore_tls_button: gtk::Button = builder.get_object("ignore_button").ok_or_else(|| format_err!("some err"))?;
        let error_details_button: gtk::Button = builder.get_object("details_button").ok_or_else(|| format_err!("some err"))?;

        let scale_factor = page.get_style_context().get_scale();

        let generic_logo_data = Resources::get("icons/feed-service-generic.svg").ok_or_else(|| format_err!("some err"))?;
        let surface = GtkUtil::create_surface_from_bytes(&generic_logo_data, 64, 64, scale_factor)?;
        logo.set_from_surface(&surface);

        let page = PasswordLogin {
            page,
            logo,
            headline,
            scale_factor,
            url_label,
            url_entry,
            user_entry,
            pass_entry,
            http_user_entry,
            http_pass_entry,
            http_revealer,
            info_bar,
            info_bar_label,
            login_button,
            ignore_tls_button,
            error_details_button,
            info_bar_close_signal: None,
            info_bar_response_signal: None,
            url_entry_signal: None,
            user_entry_signal: None,
            pass_entry_signal: None,
            http_user_entry_signal: None,
            http_pass_entry_signal: None,
            login_button_signal: None,
            ignore_tls_signal: None,
            error_details_signal: None,
        };

        Ok(page)
    }

    pub fn set_service(&mut self, info: PluginInfo) -> Result<(), Error> {
        // set Icon
        if let Some(icon) = info.icon {
            let surface = match icon {
                PluginIcon::Vector(icon) => GtkUtil::create_surface_from_bytes(&icon.data, icon.width, icon.height, self.scale_factor)?,
                PluginIcon::Pixel(icon) => GtkUtil::create_surface_from_pixelicon(&icon, self.scale_factor)?,
            };
            self.logo.set_from_surface(&surface);
        }

        // set headline
        self.headline.set_text(&format!("Please log into {} and enjoy using NewsFlash", info.name));

        // setup infobar
        self.info_bar_close_signal = Some(
            self.info_bar
                .connect_close(|info_bar| {
                    PasswordLogin::hide_info_bar(info_bar);
                })
                .to_glib(),
        );
        self.info_bar_response_signal = Some(
            self.info_bar
                .connect_response(|info_bar, response| {
                    if let ResponseType::Close = response {
                        PasswordLogin::hide_info_bar(info_bar);
                    }
                })
                .to_glib(),
        );

        if let LoginGUI::Password(pw_gui_desc) = &info.login_gui {
            // show/hide url & http-auth fields
            self.url_label.set_visible(pw_gui_desc.url);
            self.url_entry.set_visible(pw_gui_desc.url);
            self.http_revealer.set_reveal_child(pw_gui_desc.http_auth);

            // set focus to first entry
            if pw_gui_desc.url {
                self.url_entry.grab_focus();
            } else {
                self.user_entry.grab_focus();
            }

            // check if "login" should be clickable
            self.url_entry_signal = Some(self.setup_entry(&self.url_entry, &pw_gui_desc).to_glib());
            self.user_entry_signal = Some(self.setup_entry(&self.user_entry, &pw_gui_desc).to_glib());
            self.pass_entry_signal = Some(self.setup_entry(&self.pass_entry, &pw_gui_desc).to_glib());
            self.http_user_entry_signal = Some(self.setup_entry(&self.http_user_entry, &pw_gui_desc).to_glib());
            self.http_pass_entry_signal = Some(self.setup_entry(&self.http_pass_entry, &pw_gui_desc).to_glib());

            // harvest login data
            let url_entry = self.url_entry.clone();
            let user_entry = self.user_entry.clone();
            let pass_entry = self.pass_entry.clone();
            let http_user_entry = self.http_user_entry.clone();
            let http_pass_entry = self.http_pass_entry.clone();
            let pw_gui_desc = pw_gui_desc.clone();
            let plugin_id = info.id;
            self.login_button_signal = Some(
                self.login_button
                    .connect_clicked(move |_button| {
                        let url: Option<String> = if pw_gui_desc.url {
                            match url_entry.get_text() {
                                Some(url) => Some(url.as_str().to_owned()),
                                None => None,
                            }
                        } else {
                            None
                        };
                        let user = user_entry.get_text().unwrap().as_str().to_owned();
                        let password = pass_entry.get_text().unwrap().as_str().to_owned();
                        let http_user: Option<String> = if pw_gui_desc.http_auth {
                            match http_user_entry.get_text() {
                                Some(user) => Some(user.as_str().to_owned()),
                                None => None,
                            }
                        } else {
                            None
                        };
                        let http_password: Option<String> = if pw_gui_desc.http_auth {
                            match http_pass_entry.get_text() {
                                Some(pass) => Some(pass.as_str().to_owned()),
                                None => None,
                            }
                        } else {
                            None
                        };

                        let login_data = PasswordLoginData {
                            id: plugin_id.clone(),
                            url,
                            user,
                            password,
                            http_user,
                            http_password,
                        };
                        let login_data = LoginData::Password(login_data);
                        let login_data_json = serde_json::to_string(&login_data).unwrap();

                        if let Ok(main_window) = GtkUtil::get_main_window(&url_entry) {
                            if let Some(action) = main_window.lookup_action("login") {
                                let login_data_json = Variant::from(&login_data_json);
                                action.activate(Some(&login_data_json));
                            }
                        }
                    })
                    .to_glib(),
            );
        }
        Ok(())
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }

    pub fn reset(&mut self) {
        self.info_bar.set_revealed(false);
        self.info_bar.set_visible(false);
        self.url_entry.set_text("");
        self.user_entry.set_text("");
        self.pass_entry.set_text("");
        self.http_user_entry.set_text("");
        self.http_pass_entry.set_text("");

        GtkUtil::disconnect_signal(self.info_bar_close_signal, &self.info_bar);
        GtkUtil::disconnect_signal(self.info_bar_response_signal, &self.info_bar);
        GtkUtil::disconnect_signal(self.url_entry_signal, &self.url_entry);
        GtkUtil::disconnect_signal(self.user_entry_signal, &self.user_entry);
        GtkUtil::disconnect_signal(self.pass_entry_signal, &self.pass_entry);
        GtkUtil::disconnect_signal(self.http_user_entry_signal, &self.http_user_entry);
        GtkUtil::disconnect_signal(self.http_pass_entry_signal, &self.http_pass_entry);
        GtkUtil::disconnect_signal(self.login_button_signal, &self.login_button);
        GtkUtil::disconnect_signal(self.ignore_tls_signal, &self.ignore_tls_button);
        GtkUtil::disconnect_signal(self.error_details_signal, &self.error_details_button);
        self.info_bar_close_signal = None;
        self.info_bar_response_signal = None;
        self.url_entry_signal = None;
        self.user_entry_signal = None;
        self.pass_entry_signal = None;
        self.http_user_entry_signal = None;
        self.http_pass_entry_signal = None;
        self.login_button_signal = None;
        self.ignore_tls_signal = None;
        self.error_details_signal = None;
    }

    fn hide_info_bar(info_bar: &gtk::InfoBar) {
        info_bar.set_revealed(false);
        let clone = info_bar.clone();
        gtk::timeout_add(200, move || {
            clone.set_visible(false);
            gtk::Continue(false)
        });
    }

    pub fn show_error(&mut self, error: NewsFlashError) {
        GtkUtil::disconnect_signal(self.ignore_tls_signal, &self.ignore_tls_button);
        GtkUtil::disconnect_signal(self.error_details_signal, &self.error_details_button);
        self.ignore_tls_signal = None;
        self.error_details_signal = None;

        self.ignore_tls_button.set_visible(false);
        self.error_details_button.set_visible(false);

        match error.kind() {
            NewsFlashErrorKind::Login => {
                if let Some(cause) = error.cause() {
                    if let Some(api_err) = cause.downcast_ref::<FeedApiError>() {
                        match api_err.kind() {
                            FeedApiErrorKind::HTTPAuth => {
                                self.http_revealer.set_reveal_child(true);
                                self.info_bar_label.set_text("HTTP Authentication required.");
                                return;
                            }
                            FeedApiErrorKind::TLSCert => {
                                self.info_bar_label.set_text("No valid CA certificate available.");
                                self.ignore_tls_button.set_visible(true);
                                // FIXME: make button do something
                                return;
                            }
                            FeedApiErrorKind::Login | FeedApiErrorKind::Api | _ => {
                                self.info_bar_label.set_text("Failed to log in");
                            }
                        }
                    }
                }
            }
            _ => self.info_bar_label.set_text("Unknown error."),
        }

        self.error_details_button.set_visible(true);
        self.error_details_signal = Some(
            self.error_details_button
                .connect_clicked(move |button| {
                    let parent = GtkUtil::get_main_window(button).unwrap();
                    let _dialog = ErrorDialog::new(&error, &parent).unwrap();
                })
                .to_glib(),
        );
        self.info_bar.set_visible(true);
        self.info_bar.set_revealed(true);
    }

    fn setup_entry(&self, entry: &gtk::Entry, gui_desc: &PasswordLoginGUI) -> SignalHandlerId {
        let entry = entry.clone();
        let gui_desc = gui_desc.clone();
        let button = self.login_button.clone();
        let url_entry = self.url_entry.clone();
        let user_entry = self.user_entry.clone();
        let pass_entry = self.pass_entry.clone();
        let http_user_entry = self.http_user_entry.clone();
        let http_pass_entry = self.http_pass_entry.clone();

        entry.connect_property_text_notify(move |_entry| {
            if gui_desc.url && GtkUtil::is_entry_emty(&url_entry) {
                button.set_sensitive(false);
                return;
            }
            if GtkUtil::is_entry_emty(&user_entry) {
                button.set_sensitive(false);
                return;
            }
            if GtkUtil::is_entry_emty(&pass_entry) {
                button.set_sensitive(false);
                return;
            }
            if gui_desc.http_auth {
                if GtkUtil::is_entry_emty(&http_user_entry) {
                    button.set_sensitive(false);
                    return;
                }
                if GtkUtil::is_entry_emty(&http_pass_entry) {
                    button.set_sensitive(false);
                    return;
                }
            }

            button.set_sensitive(true);
        })
    }
}
