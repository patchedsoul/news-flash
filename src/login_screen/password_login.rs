use super::error::{LoginScreenError, LoginScreenErrorKind};
use crate::app::Action;
use crate::error_dialog::ErrorDialog;
use crate::i18n::{i18n, i18n_f};
use crate::util::{BuilderHelper, GtkUtil, Util};
use failure::{Fail, ResultExt};
use glib::{clone, signal::SignalHandlerId, source::Continue, translate::ToGlib, Sender};
use gtk::{
    self, Box, Button, ButtonExt, Entry, EntryExt, Image, ImageExt, InfoBar, InfoBarExt, Label, LabelExt, ResponseType,
    Revealer, RevealerExt, WidgetExt,
};
use news_flash::models::{
    LoginData, LoginGUI, PasswordLogin as PasswordLoginData, PasswordLoginGUI, PluginIcon, PluginInfo,
};
use news_flash::{FeedApiError, FeedApiErrorKind, NewsFlashError, NewsFlashErrorKind};
use parking_lot::RwLock;

#[derive(Debug)]
pub struct PasswordLogin {
    sender: Sender<Action>,
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
    info_bar_close_signal: RwLock<Option<usize>>,
    info_bar_response_signal: RwLock<Option<usize>>,
    url_entry_signal: RwLock<Option<usize>>,
    user_entry_signal: RwLock<Option<usize>>,
    pass_entry_signal: RwLock<Option<usize>>,
    http_user_entry_signal: RwLock<Option<usize>>,
    http_pass_entry_signal: RwLock<Option<usize>>,
    login_button_signal: RwLock<Option<usize>>,
    ignore_tls_signal: RwLock<Option<usize>>,
    error_details_signal: RwLock<Option<usize>>,
}

impl PasswordLogin {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let page = builder.get::<Box>("password_login");
        let logo = builder.get::<Image>("logo");
        let headline = builder.get::<Label>("headline");
        let url_label = builder.get::<Label>("url_label");
        let url_entry = builder.get::<Entry>("url_entry");
        let user_entry = builder.get::<Entry>("user_entry");
        let pass_entry = builder.get::<Entry>("pass_entry");
        let http_user_entry = builder.get::<Entry>("http_user_entry");
        let http_pass_entry = builder.get::<Entry>("http_pass_entry");
        let http_revealer = builder.get::<Revealer>("http_auth_revealer");
        let login_button = builder.get::<Button>("login_button");
        let info_bar = builder.get::<InfoBar>("pw_info_bar");
        let info_bar_label = builder.get::<Label>("pw_info_bar_label");
        let ignore_tls_button = builder.get::<Button>("ignore_button");
        let error_details_button = builder.get::<Button>("pw_details_button");
        let back_button = builder.get::<Button>("pass_back_button");

        back_button.connect_clicked(clone!(@strong sender => @default-panic, move |_button| {
            Util::send(&sender, Action::ShowWelcomePage);
        }));

        let scale_factor = GtkUtil::get_scale(&page);
        let surface = GtkUtil::create_surface_from_icon_name("feed-service-generic", 64, scale_factor);
        logo.set_from_surface(Some(&surface));

        ignore_tls_button.connect_clicked(clone!(@weak info_bar, @strong sender => @default-panic, move |button| {
            Util::send(&sender, Action::IgnoreTLSErrors);
            PasswordLogin::hide_info_bar(&info_bar);
            button.set_visible(false);
        }));

        PasswordLogin {
            sender,
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
            info_bar_close_signal: RwLock::new(None),
            info_bar_response_signal: RwLock::new(None),
            url_entry_signal: RwLock::new(None),
            user_entry_signal: RwLock::new(None),
            pass_entry_signal: RwLock::new(None),
            http_user_entry_signal: RwLock::new(None),
            http_pass_entry_signal: RwLock::new(None),
            login_button_signal: RwLock::new(None),
            ignore_tls_signal: RwLock::new(None),
            error_details_signal: RwLock::new(None),
        }
    }

    pub fn set_service(&self, info: PluginInfo) -> Result<(), LoginScreenError> {
        // set Icon
        if let Some(icon) = info.icon {
            let surface = match icon {
                PluginIcon::Vector(icon) => {
                    GtkUtil::create_surface_from_bytes(&icon.data, icon.width, icon.height, self.scale_factor)
                        .context(LoginScreenErrorKind::Icon)?
                }
                PluginIcon::Pixel(icon) => GtkUtil::create_surface_from_pixelicon(&icon, self.scale_factor)
                    .context(LoginScreenErrorKind::Icon)?,
            };
            self.logo.set_from_surface(Some(&surface));
        }

        // set headline
        self.headline
            .set_text(&i18n_f("Please log into {} and enjoy using NewsFlash", &[&info.name]));

        // setup infobar
        self.info_bar_close_signal.write().replace(
            self.info_bar
                .connect_close(|info_bar| {
                    PasswordLogin::hide_info_bar(info_bar);
                })
                .to_glib() as usize,
        );
        self.info_bar_response_signal.write().replace(
            self.info_bar
                .connect_response(|info_bar, response| {
                    if let ResponseType::Close = response {
                        PasswordLogin::hide_info_bar(info_bar);
                    }
                })
                .to_glib() as usize,
        );

        if let LoginGUI::Password(pw_gui_desc) = &info.login_gui {
            // show/hide url & http-auth fields
            self.url_label.set_visible(pw_gui_desc.url);
            self.url_entry.set_visible(pw_gui_desc.url);
            self.http_revealer.set_reveal_child(false);

            // set focus to first entry
            if pw_gui_desc.url {
                self.url_entry.grab_focus();
            } else {
                self.user_entry.grab_focus();
            }

            // check if 'login' should be clickable
            self.url_entry_signal
                .write()
                .replace(self.setup_entry(&self.url_entry, &pw_gui_desc).to_glib() as usize);
            self.user_entry_signal
                .write()
                .replace(self.setup_entry(&self.user_entry, &pw_gui_desc).to_glib() as usize);
            self.pass_entry_signal
                .write()
                .replace(self.setup_entry(&self.pass_entry, &pw_gui_desc).to_glib() as usize);
            self.http_user_entry_signal
                .write()
                .replace(self.setup_entry(&self.http_user_entry, &pw_gui_desc).to_glib() as usize);
            self.http_pass_entry_signal
                .write()
                .replace(self.setup_entry(&self.http_pass_entry, &pw_gui_desc).to_glib() as usize);

            // harvest login data
            self.login_button_signal.write().replace(
                self.login_button
                    .connect_clicked(clone!(
                        @weak self.url_entry as url_entry,
                        @weak self.user_entry as user_entry,
                        @weak self.pass_entry as pass_entry,
                        @weak self.http_user_entry as http_user_entry,
                        @weak self.http_pass_entry as http_pass_entry,
                        @weak self.http_revealer as http_revealer,
                        @strong pw_gui_desc,
                        @strong info.id as plugin_id,
                        @strong self.sender as sender => @default-panic, move |_button|
                    {
                        let url: Option<String> = if pw_gui_desc.url {
                            Some(url_entry.get_text().as_str().to_owned())
                        } else {
                            None
                        };
                        let user = user_entry
                            .get_text()
                            .as_str()
                            .to_owned();
                        let password = pass_entry
                            .get_text()
                            .as_str()
                            .to_owned();
                        let http_user: Option<String> = if http_revealer.get_child_revealed() {
                            if http_user_entry.get_text().is_empty() {
                                None
                            } else {
                                Some(http_user_entry.get_text().as_str().to_owned())
                            }
                        } else {
                            None
                        };
                        let http_password: Option<String> = if http_revealer.get_child_revealed() {
                            if http_pass_entry.get_text().is_empty() {
                                None
                            } else {
                                Some(http_pass_entry.get_text().as_str().to_owned())
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
                        Util::send(&sender, Action::Login(login_data));
                    }))
                    .to_glib() as usize,
            );

            return Ok(());
        }

        Err(LoginScreenErrorKind::LoginGUI.into())
    }

    pub fn reset(&self) {
        self.info_bar.set_revealed(false);
        self.info_bar.set_visible(false);
        self.url_entry.set_text("");
        self.user_entry.set_text("");
        self.pass_entry.set_text("");
        self.http_user_entry.set_text("");
        self.http_pass_entry.set_text("");

        GtkUtil::disconnect_signal(*self.info_bar_close_signal.read(), &self.info_bar);
        GtkUtil::disconnect_signal(*self.info_bar_response_signal.read(), &self.info_bar);
        GtkUtil::disconnect_signal(*self.url_entry_signal.read(), &self.url_entry);
        GtkUtil::disconnect_signal(*self.user_entry_signal.read(), &self.user_entry);
        GtkUtil::disconnect_signal(*self.pass_entry_signal.read(), &self.pass_entry);
        GtkUtil::disconnect_signal(*self.http_user_entry_signal.read(), &self.http_user_entry);
        GtkUtil::disconnect_signal(*self.http_pass_entry_signal.read(), &self.http_pass_entry);
        GtkUtil::disconnect_signal(*self.login_button_signal.read(), &self.login_button);
        GtkUtil::disconnect_signal(*self.ignore_tls_signal.read(), &self.ignore_tls_button);
        GtkUtil::disconnect_signal(*self.error_details_signal.read(), &self.error_details_button);
        self.info_bar_close_signal.write().take();
        self.info_bar_response_signal.write().take();
        self.url_entry_signal.write().take();
        self.user_entry_signal.write().take();
        self.pass_entry_signal.write().take();
        self.http_user_entry_signal.write().take();
        self.http_pass_entry_signal.write().take();
        self.login_button_signal.write().take();
        self.ignore_tls_signal.write().take();
        self.error_details_signal.write().take();
    }

    fn hide_info_bar(info_bar: &gtk::InfoBar) {
        info_bar.set_revealed(false);
        gtk::timeout_add(
            200,
            clone!(@weak info_bar => @default-panic, move || {
                info_bar.set_visible(false);
                Continue(false)
            }),
        );
    }

    pub fn show_error(&self, error: NewsFlashError) {
        GtkUtil::disconnect_signal(*self.ignore_tls_signal.read(), &self.ignore_tls_button);
        GtkUtil::disconnect_signal(*self.error_details_signal.read(), &self.error_details_button);
        self.ignore_tls_signal.write().take();
        self.error_details_signal.write().take();

        self.ignore_tls_button.set_visible(false);
        self.error_details_button.set_visible(false);

        match error.kind() {
            NewsFlashErrorKind::Login => {
                if let Some(cause) = error.cause() {
                    if let Some(api_err) = cause.downcast_ref::<FeedApiError>() {
                        match api_err.kind() {
                            FeedApiErrorKind::HTTPAuth => {
                                self.http_revealer.set_reveal_child(true);
                                self.login_button.set_sensitive(false);
                                self.info_bar_label.set_text(&i18n("HTTP Authentication required."));
                            }
                            FeedApiErrorKind::TLSCert => {
                                self.info_bar_label
                                    .set_text(&i18n("No valid CA certificate available."));
                                self.ignore_tls_button.set_visible(true);
                            }
                            FeedApiErrorKind::Login | FeedApiErrorKind::Api | _ => {
                                self.info_bar_label.set_text(&i18n("Failed to log in"));
                            }
                        }
                    }
                }
            }
            _ => self.info_bar_label.set_text(&i18n("Unknown error.")),
        }

        self.error_details_button.set_visible(true);
        self.error_details_signal.write().replace(
            self.error_details_button
                .connect_clicked(move |button| {
                    let parent = GtkUtil::get_main_window(button)
                        .expect("MainWindow is not a parent of password login error details button.");
                    let _dialog = ErrorDialog::new(&error, &parent);
                })
                .to_glib() as usize,
        );
        self.info_bar.set_visible(true);
        self.info_bar.set_revealed(true);
    }

    pub fn fill(&self, data: PasswordLoginData) {
        self.info_bar.set_revealed(false);
        self.info_bar.set_visible(false);
        self.url_entry.set_text("");
        self.user_entry.set_text("");
        self.pass_entry.set_text("");
        self.http_user_entry.set_text("");
        self.http_pass_entry.set_text("");

        if let Some(url) = &data.url {
            self.url_entry.set_text(url);
        }
        self.user_entry.set_text(&data.user);
        self.pass_entry.set_text(&data.password);

        if let Some(http_user) = &data.http_user {
            self.http_user_entry.set_text(http_user);
            self.http_revealer.set_reveal_child(true);
        }
        if let Some(http_password) = &data.http_password {
            self.http_pass_entry.set_text(http_password);
        }
    }

    fn setup_entry(&self, entry: &Entry, gui_desc: &PasswordLoginGUI) -> SignalHandlerId {
        entry.connect_property_text_notify(clone!(
            @weak self.login_button as button,
            @weak self.url_entry as url_entry,
            @weak self.user_entry as user_entry,
            @weak self.pass_entry as pass_entry,
            @weak self.http_user_entry as http_user_entry,
            @weak self.http_revealer as http_revealer,
            @strong gui_desc => @default-panic, move |_entry|
        {
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
            if http_revealer.get_child_revealed() && GtkUtil::is_entry_emty(&http_user_entry) {
                button.set_sensitive(false);
                return;
            }

            button.set_sensitive(true);
        }))
    }
}
