use gtk::{
    self,
    ImageExt,
    WidgetExt,
    StyleContextExt,
    ButtonExt,
    LabelExt,
    RevealerExt,
    EntryExt,
    InfoBarExt,
    ResponseType,
};
use crate::gtk_util::GtkUtil;
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use news_flash::models::{
    PluginInfo,
    PluginIcon,
    LoginGUI,
    PasswordLoginGUI,
};


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
    login_button: gtk::Button,
}

impl PasswordLogin {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/password_login.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("password_login").ok_or(format_err!("some err"))?;
        let logo : gtk::Image = builder.get_object("logo").ok_or(format_err!("some err"))?;
        let headline : gtk::Label = builder.get_object("headline").ok_or(format_err!("some err"))?;
        let url_label : gtk::Label = builder.get_object("url_label").ok_or(format_err!("some err"))?;
        let url_entry : gtk::Entry = builder.get_object("url_entry").ok_or(format_err!("some err"))?;
        let user_entry : gtk::Entry = builder.get_object("user_entry").ok_or(format_err!("some err"))?;
        let pass_entry : gtk::Entry = builder.get_object("pass_entry").ok_or(format_err!("some err"))?;
        let http_user_entry : gtk::Entry = builder.get_object("http_user_entry").ok_or(format_err!("some err"))?;
        let http_pass_entry : gtk::Entry = builder.get_object("http_pass_entry").ok_or(format_err!("some err"))?;
        let http_revealer : gtk::Revealer = builder.get_object("http_auth_revealer").ok_or(format_err!("some err"))?;
        let login_button : gtk::Button = builder.get_object("login_button").ok_or(format_err!("some err"))?;
        let info_bar : gtk::InfoBar = builder.get_object("info_bar").ok_or(format_err!("some err"))?;

        info_bar.connect_close(|bar| {
            PasswordLogin::hide_info_bar(bar);
        });
        info_bar.connect_response(|bar, response| {
            let response = ResponseType::from(response);
            match response {
                ResponseType::Close => {
                    PasswordLogin::hide_info_bar(bar);
                },
                _ => {},
            }
        });

        let ctx = page.get_style_context().ok_or(format_err!("some err"))?;
        let scale = ctx.get_scale();

        let generic_logo_data = Resources::get("icons/feed_service_generic.svg").ok_or(format_err!("some err"))?;
        let surface = GtkUtil::create_surface_from_svg(&generic_logo_data, 64, 64, scale)?;
        logo.set_from_surface(&surface);

        let page = PasswordLogin {
            page: page,
            logo: logo,
            headline: headline,
            scale_factor: scale,
            url_label: url_label,
            url_entry: url_entry,
            user_entry: user_entry,
            pass_entry: pass_entry,
            http_user_entry: http_user_entry,
            http_pass_entry: http_pass_entry,
            http_revealer: http_revealer,
            info_bar: info_bar,
            login_button: login_button,
        };

        Ok(page)
    }

    pub fn set_service(&self, info: PluginInfo, gui_desc: LoginGUI) -> Result<(), Error> {
        
        // set Icon
        if let Some(icon) = info.icon {
            let surface = match icon {
                PluginIcon::Vector(icon) => {
                    GtkUtil::create_surface_from_svg(&icon.data, icon.width, icon.height, self.scale_factor)?
                },
                PluginIcon::Pixel(icon) => {
                    GtkUtil::create_surface_from_bitmap(icon, self.scale_factor)?
                },
            };
            self.logo.set_from_surface(&surface);
        }

        // set headline
        self.headline.set_text(&format!("Please log into {} and enjoy using NewsFlash", info.name));


        if let LoginGUI::Password(pw_gui_desc) = &gui_desc {

             // show/hide url & http-auth fields
            self.url_label.set_visible(pw_gui_desc.url);
            self.url_entry.set_visible(pw_gui_desc.url);
            self.http_revealer.set_reveal_child(pw_gui_desc.http_auth);

            // set focus to first entry
            if pw_gui_desc.url {
                self.url_entry.grab_focus();
            }
            else {
                self.user_entry.grab_focus();
            }

            // check if "login" should be clickable
            self.setup_entry(&self.url_entry,       &pw_gui_desc);
            self.setup_entry(&self.user_entry,      &pw_gui_desc);
            self.setup_entry(&self.pass_entry,      &pw_gui_desc);
            self.setup_entry(&self.http_pass_entry, &pw_gui_desc);
            self.setup_entry(&self.http_user_entry, &pw_gui_desc);
        }
        Ok(())
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }

    pub fn reset(&self) {
        self.url_entry.set_text("");
        self.user_entry.set_text("");
        self.pass_entry.set_text("");
        self.http_user_entry.set_text("");
        self.http_pass_entry.set_text("");
    }

    fn hide_info_bar(bar: &gtk::InfoBar) {
        bar.set_revealed(false);
        let clone = bar.clone();
        gtk::timeout_add(200, move || {
            clone.set_visible(false);
            gtk::Continue(false)
        });
    }

    // fn show_info_bar(bar: &gtk::InfoBar) {
    //     bar.set_visible(true);
    //     bar.set_revealed(true);
    // }

    fn setup_entry(
        &self,
        entry: &gtk::Entry,
        gui_desc: &PasswordLoginGUI,
    ) {
        let entry = entry.clone();
        let gui_desc = gui_desc.clone();
        let button = self.login_button.clone();
        let url_entry = self.url_entry.clone();
        let user_entry = self.user_entry.clone();
        let pass_entry = self.pass_entry.clone();
        let http_user_entry = self.http_user_entry.clone();
        let http_pass_entry = self.http_pass_entry.clone();

        entry.connect_property_text_notify(move |_entry| {
            if gui_desc.url
            && GtkUtil::is_entry_emty(&url_entry) {
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
        });
    }
}