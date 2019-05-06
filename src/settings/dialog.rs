use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkHandle};
use super::theme_chooser::ThemeChooser;
use gtk::{Dialog, DialogExt, Window, GtkWindowExt, GtkWindowExtManual, Inhibit, FontButton, FontButtonExt, FontChooserExt,
    Label, ListBox, ListBoxExt, Stack, StackExt, Switch, SwitchExt, WidgetExt};
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
        
        let settings_1 = settings.clone();
        let settings_2 = settings.clone();
        let settings_3 = settings.clone();
        let settings_4 = settings.clone();
        let settings_5 = settings.clone();

        let main_window = window.clone();
        let article_list_settings = builder.get::<ListBox>("article_list_settings");
        article_list_settings.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if "order" == row_name {
                    let order = settings_1.borrow().get_article_list_order();
                    settings_1.borrow_mut().set_article_list_order(order.invert()).unwrap();
                    Self::set_article_list_order_stack(&settings_1, &article_list_order_stack);
                    if let Some(action) = main_window.lookup_action("update-article-list") {
                        action.activate(None);
                    }
                }
            }
        });

        let dialog_window = dialog.clone();
        let main_window = window.clone();
        let article_view_settings = builder.get::<ListBox>("article_view_settings");
        article_view_settings.connect_row_activated(move |_list, row| {
            if let Some(row_name) = row.get_name() {
                if "theme" == row_name {
                    let main_window = main_window.clone();
                    let theme_chooser = ThemeChooser::new(&dialog_window, &settings_2);
                    theme_chooser.widget().connect_close(move |_dialog| {
                        if let Some(action) = main_window.lookup_action("redraw-article") {
                            action.activate(None);
                        }
                    });
                    theme_chooser.widget().present();
                }
            }
        });

        let main_window = window.clone();
        let allow_selection_switch = builder.get::<Switch>("allow_selection_switch");
        allow_selection_switch.set_state(settings.borrow().get_article_view_allow_select());
        allow_selection_switch.connect_state_set(move |_switch, is_set| {
            settings_3.borrow_mut().set_article_view_allow_select(is_set).unwrap();
            if let Some(action) = main_window.lookup_action("redraw-article") {
                action.activate(None);
            }
            Inhibit(false)
        });

        let main_window = window.clone();
        let font_label = builder.get::<Label>("font_label");
        let font_button = builder.get::<FontButton>("font_button");
        if let Some(font) = settings.borrow().get_article_view_font() {
            font_button.set_font(&font);
        }
        font_button.connect_font_set(move |button| {
            let font =  match button.get_font() {
                Some(font) => Some(font.to_string()),
                None => None,
            };
            settings_5.borrow_mut().set_article_view_font(font).unwrap();
            if let Some(action) = main_window.lookup_action("redraw-article") {
                action.activate(None);
            }
        });


        let main_window = window.clone();
        let use_system_font_switch = builder.get::<Switch>("use_system_font_switch");
        let have_custom_font = match settings.borrow().get_article_view_font() {
            Some(_) => true,
            None => false,
        };
        use_system_font_switch.set_state(!have_custom_font);
        font_button.set_sensitive(have_custom_font);
        font_label.set_sensitive(have_custom_font);
        use_system_font_switch.connect_state_set(move |_switch, is_set| {
            let font = if is_set {
                None
            } else {
                if let Some(font_name) = font_button.get_font() {
                    Some(font_name.to_string())
                } else {
                    None
                }
            };
            font_button.set_sensitive(!is_set);
            font_label.set_sensitive(!is_set);
            settings_4.borrow_mut().set_article_view_font(font).unwrap();
            if let Some(action) = main_window.lookup_action("redraw-article") {
                action.activate(None);
            }
            Inhibit(false)
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