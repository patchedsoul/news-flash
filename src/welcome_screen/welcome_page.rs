use super::service_row::ServiceRow;
use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandleMap, GtkUtil};
use failure::Error;
use gio::{ActionExt, ActionMapExt};
use glib::Variant;
use gtk::{Box, ListBox, ListBoxExt, ListBoxRowExt};
use news_flash::models::{LoginGUI, PluginID};
use news_flash::NewsFlash;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct WelcomePage {
    page: gtk::Box,
    list: gtk::ListBox,
    services: GtkHandleMap<i32, (PluginID, LoginGUI)>,
}

impl WelcomePage {
    pub fn new(builder: &BuilderHelper) -> Result<Self, Error> {
        let page = builder.get::<Box>("welcome_page");
        let list = builder.get::<ListBox>("list");

        let mut page = WelcomePage {
            page,
            list,
            services: gtk_handle!(HashMap::new()),
        };

        page.populate()?;
        page.connect_signals();

        Ok(page)
    }

    fn populate(&mut self) -> Result<(), Error> {
        let services = NewsFlash::list_backends();
        for (index, (id, api_meta)) in services.iter().enumerate() {
            let row = ServiceRow::new(api_meta.clone())?;
            self.list.insert(&row.widget(), index as i32);
            self.services
                .borrow_mut()
                .insert(index as i32, (id.clone(), api_meta.login_gui.clone()));
        }
        Ok(())
    }

    fn connect_signals(&self) {
        let main_window = GtkUtil::get_main_window(&self.page).unwrap();
        let services = self.services.clone();
        self.list.connect_row_activated(move |_list, row| {
            if let Some((id, login_desc)) = services.borrow().get(&row.get_index()) {
                let id = Variant::from(id.to_str());
                match login_desc {
                    LoginGUI::OAuth(_) => {
                        if let Some(action) = main_window.lookup_action("show-oauth-page") {
                            action.activate(Some(&id));
                        }
                    }
                    LoginGUI::Password(_) => {
                        if let Some(action) = main_window.lookup_action("show-pw-page") {
                            action.activate(Some(&id));
                        }
                    },
                    LoginGUI::None => {
                        // FIXME: trigger "login" action directly
                    },
                };
            }
        });
    }
}
