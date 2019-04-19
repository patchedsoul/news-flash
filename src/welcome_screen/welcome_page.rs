use crate::util::{GTK_BUILDER_ERROR, GTK_RESOURCE_FILE_ERROR};
use super::service_row::ServiceRow;
use crate::gtk_handle;
use crate::util::GtkHandleMap;
use crate::Resources;
use failure::Error;
use gio::{ActionExt, ActionMapExt};
use glib::Variant;
use gtk::{self, ApplicationWindow, ListBoxExt, ListBoxRowExt};
use news_flash::models::{LoginGUI, PluginID};
use news_flash::NewsFlash;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str;

#[derive(Clone, Debug)]
pub struct WelcomePage {
    page: gtk::Box,
    list: gtk::ListBox,
    services: GtkHandleMap<i32, (PluginID, LoginGUI)>,
}

impl WelcomePage {
    pub fn new(window: &ApplicationWindow) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/welcome_page.ui").expect(GTK_RESOURCE_FILE_ERROR);
        let ui_string = str::from_utf8(ui_data.as_ref()).expect(GTK_RESOURCE_FILE_ERROR);

        let builder = gtk::Builder::new_from_string(ui_string);
        let page: gtk::Box = builder.get_object("welcome_page").expect(GTK_BUILDER_ERROR);
        let list: gtk::ListBox = builder.get_object("list").expect(GTK_BUILDER_ERROR);

        let mut page = WelcomePage {
            page,
            list,
            services: gtk_handle!(HashMap::new()),
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
                    }
                    LoginGUI::Password(_) => {
                        if let Some(action) = main_window.lookup_action("show-pw-page") {
                            action.activate(Some(&id));
                        }
                    }
                };
            }
        });
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }
}
