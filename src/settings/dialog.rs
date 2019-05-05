use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkHandle};
use gtk::{Dialog, Window, GtkWindowExt, ListBox, ListBoxExt, Stack, StackExt, WidgetExt};
use glib::{object::IsA};
use gio::{ActionExt, ActionMapExt};
use news_flash::models::ArticleOrder;


pub struct SettingsDialog {
    widget: Dialog,
}

impl SettingsDialog {
    pub fn new<W: IsA<Window> + GtkWindowExt + ActionMapExt>(window: &W, settings: &GtkHandle<Settings>) -> Self {
        let builder = BuilderHelper::new("settings");

        let dialog = builder.get::<Dialog>("dialog");
        dialog.set_transient_for(window);

        let article_list_order_stack = builder.get::<Stack>("article_list_order_stack");
        Self::set_article_list_order_stack(settings, &article_list_order_stack);
        
        let settings = settings.clone();
        let main_window = window.clone();
        let article_list_settings = builder.get::<ListBox>("article_list_settings");
        article_list_settings.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if "order" == row_name {
                    let order = settings.borrow().get_article_list_order();
                    settings.borrow_mut().set_article_list_order(order.invert()).unwrap();
                    Self::set_article_list_order_stack(&settings, &article_list_order_stack);
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
                }
            }
        });

        SettingsDialog {
            widget: dialog,
        }
    }

    pub fn widget(&self) -> Dialog {
        self.widget.clone()
    }

    fn set_article_list_order_stack(settings: &GtkHandle<Settings>, article_list_order_stack: &Stack) {
        match settings.borrow().get_article_list_order() {
            ArticleOrder::NewestFirst => article_list_order_stack.set_visible_child_name("new"),
            ArticleOrder::OldestFirst => article_list_order_stack.set_visible_child_name("old"),
        }
    }
}