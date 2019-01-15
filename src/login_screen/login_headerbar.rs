use failure::{
    Error,
    format_err,
};
use gtk::{
    ButtonExt,
};
use gio::{
    ActionExt,
    ActionMapExt,
};
use crate::Resources;
use std::str;

#[derive(Clone, Debug)]
pub struct LoginHeaderbar {
    widget: gtk::HeaderBar,
}

impl LoginHeaderbar {
    pub fn new(main_window: &gtk::ApplicationWindow) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/login_headerbar.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let headerbar : gtk::HeaderBar = builder.get_object("login_headerbar").ok_or(format_err!("some err"))?;
        let button : gtk::Button = builder.get_object("back_button").ok_or(format_err!("some err"))?;
        let main_window = main_window.clone();
        button.connect_clicked(move |_button| {
            if let Some(action) = main_window.lookup_action("show-welcome-page") {
                action.activate(None);
            }
        });

        Ok(LoginHeaderbar{
            widget: headerbar,
        })
    }

    pub fn widget(&self) -> gtk::HeaderBar {
        self.widget.clone()
    }
}