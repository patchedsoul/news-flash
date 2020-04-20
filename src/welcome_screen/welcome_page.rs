use super::service_row::ServiceRow;
use crate::app::Action;
use crate::util::{BuilderHelper, Util};
use glib::{clone, Sender};
use gtk::{Box, ListBox, ListBoxExt, ListBoxRowExt};
use news_flash::models::{LoginData, LoginGUI, PluginID};
use news_flash::NewsFlash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct WelcomePage {
    page: gtk::Box,
    list: gtk::ListBox,
    services: Arc<RwLock<HashMap<i32, (PluginID, LoginGUI)>>>,
}

impl WelcomePage {
    pub fn new(builder: &BuilderHelper, sender: Sender<Action>) -> Self {
        let page = builder.get::<Box>("welcome_page");
        let list = builder.get::<ListBox>("list");

        let page = WelcomePage {
            page,
            list,
            services: Arc::new(RwLock::new(HashMap::new())),
        };

        page.populate();
        page.connect_signals(sender);

        page
    }

    fn populate(&self) {
        let services = NewsFlash::list_backends();
        for (index, (id, api_meta)) in services.iter().enumerate() {
            let row = ServiceRow::new(api_meta.clone());
            self.list.insert(&row.widget(), index as i32);
            self.services
                .write()
                .insert(index as i32, (id.clone(), api_meta.login_gui.clone()));
        }
    }

    fn connect_signals(&self, sender: Sender<Action>) {
        self.list.connect_row_activated(
            clone!(@strong sender, @strong self.services as services => @default-panic, move |_list, row| {
                if let Some((id, login_desc)) = services.read().get(&row.get_index()) {
                    match login_desc {
                        LoginGUI::OAuth(_) => {
                            Util::send(&sender, Action::ShowOauthLogin(id.clone()));
                        }
                        LoginGUI::Password(_) => {
                            Util::send(&sender, Action::ShowPasswordLogin(id.clone(), None));
                        }
                        LoginGUI::None => {
                            Util::send(&sender, Action::Login(LoginData::None(id.clone())));
                        }
                    };
                }
            }),
        );
    }
}
