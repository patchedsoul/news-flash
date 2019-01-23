use gtk::{
    self,
    ListBoxExt,
    ListBoxRowExt,
    ApplicationWindow,
};
use glib::{
    Variant,
};
use gio::{
    ActionExt,
    ActionMapExt,
};
use crate::Resources;
use failure::Error;
use failure::format_err;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use news_flash::NewsFlash;
use news_flash::models::{
    PluginID,
    LoginGUI,
};
use super::service_row::ServiceRow;
use std::collections::HashMap;
use crate::main_window::GtkHandleMap;

#[derive(Clone, Debug)]
pub struct WelcomePage {
    page: gtk::Box,
    list: gtk::ListBox,
    services: GtkHandleMap<i32, (PluginID, LoginGUI)>,
}

impl WelcomePage {
    pub fn new(window: &ApplicationWindow) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/welcome_page.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("welcome_page").ok_or(format_err!("some err"))?;
        let list : gtk::ListBox = builder.get_object("list").ok_or(format_err!("some err"))?;

        let mut page = WelcomePage {
            page: page,
            list: list,
            services: Rc::new(RefCell::new(HashMap::new())),
        };

        page.populate()?;
        page.connect_signals(window);

        Ok(page)
    }

    fn populate(&mut self) -> Result<(), Error> {
        let services = NewsFlash::list_backends();
        for (index, (id, api_meta)) in services.iter().enumerate() {
            let row = ServiceRow::new(api_meta.clone())?;
            self.list.insert(&row.widget(), index as i32);
            self.services.borrow_mut().insert(index as i32, (id.clone(), api_meta.login_gui.clone()));
        }
        Ok(())
    }

    fn connect_signals(&self, window: &ApplicationWindow) {
        let main_window = window.clone();
        let services = self.services.clone();
        self.list.connect_row_activated(move |_list, row| {
            if let Some((id, login_desc)) = services.borrow().get(&row.get_index()) {
                let id = Variant::from(id.to_str());
                match login_desc {
                    LoginGUI::OAuth(_) => {
                        if let Some(action) = main_window.lookup_action("show-oauth-page") {
                            action.activate(Some(&id));
                        }
                    },
                    LoginGUI::Password(_) => {
                        if let Some(action) = main_window.lookup_action("show-pw-page") {
                            action.activate(Some(&id));
                        }
                    },
                };
            }
        });
    }

    pub fn widget(&self,) -> gtk::Box {
        self.page.clone()
    }
}