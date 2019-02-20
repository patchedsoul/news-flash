mod content_header;

pub use self::content_header::ContentHeader;

use failure::Error;
use failure::format_err;
use crate::Resources;
use std::str;
use gtk::{
    Builder,
    BoxExt,
    PanedExt,
};
use glib::{
    Variant,
};
use gio::{
    ActionExt,
    ActionMapExt,
};
use crate::gtk_util::GtkUtil;
use crate::sidebar::{
    SideBar,
    FeedListTree,
};
use news_flash::models::{
    PluginID,
};

const SIDEBAR_PANED_DEFAULT_POS: i32 = 220;

pub struct ContentPage {
    page: gtk::Box,
    paned: gtk::Paned,
    sidebar: SideBar,
}

impl ContentPage {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/content_page.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let page : gtk::Box = builder.get_object("page").ok_or(format_err!("some err"))?;
        let feed_list_box : gtk::Box = builder.get_object("feedlist_box").ok_or(format_err!("some err"))?;
        let paned : gtk::Paned = builder.get_object("paned_lists_article_view").ok_or(format_err!("some err"))?;
        let sidebar_paned : gtk::Paned = builder.get_object("paned_lists").ok_or(format_err!("some err"))?;
        sidebar_paned.set_position(SIDEBAR_PANED_DEFAULT_POS);

        paned.connect_property_position_notify(|paned| {
            if let Ok(main_window) = GtkUtil::get_main_window(paned) {
                if let Some(action) = main_window.lookup_action("sync-paned") {
                    let pos = Variant::from(&paned.get_position());
                    action.activate(Some(&pos));
                }
            }
        });
        
        let sidebar = SideBar::new()?;

        feed_list_box.pack_start(&sidebar.widget(), false, true, 0);

        Ok(ContentPage {
            page: page,
            paned: paned,
            sidebar: sidebar,
        })
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }

    pub fn update_feedlist(&mut self, tree: FeedListTree) {
        self.sidebar.update_feedlist(tree);
    }

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), Error> {
        self.sidebar.set_service(id, user_name)?;
        Ok(())
    }

    pub fn set_paned(&self, pos: i32) {
        self.paned.set_position(pos);
    }
}