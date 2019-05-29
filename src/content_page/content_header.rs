use super::header_selection::HeaderSelection;
use crate::util::{BuilderHelper, GtkUtil};
use gio::{ActionExt, ActionMapExt, Menu, MenuItem};
use glib::Variant;
use gdk::EventType;
use gtk::{
    Button, ButtonExt, Label, LabelExt, Stack, StackExt, ToggleButton,
    ToggleButtonExt, WidgetExt, MenuButton, MenuButtonExt, Inhibit,
};

pub struct ContentHeader {
    update_stack: Stack,
    update_button: Button,
    //search_button: Button,
    all_button: ToggleButton,
    unread_button: ToggleButton,
    marked_button: ToggleButton,
    more_actions_button: MenuButton,
    mode_switch_button_label: Label,
}

impl ContentHeader {
    pub fn new(builder: &BuilderHelper) -> Self {
        let all_button = builder.get::<ToggleButton>("all_button");
        let unread_button = builder.get::<ToggleButton>("unread_button");
        let marked_button = builder.get::<ToggleButton>("marked_button");
        let update_button = builder.get::<Button>("update_button");
        let update_stack = builder.get::<Stack>("update_stack");
        let menu_button = builder.get::<MenuButton>("menu_button");
        let more_actions_button = builder.get::<MenuButton>("more_actions_button");
        //let search_button = builder.get::<Button>("search_button");
        let mode_button = builder.get::<MenuButton>("mode_switch_button");
        let mode_switch_button_label = builder.get::<Label>("mode_switch_button_label");

        Self::setup_linked_button(&all_button, &unread_button, &marked_button, HeaderSelection::All);
        Self::setup_linked_button(&unread_button, &all_button, &marked_button, HeaderSelection::Unread);
        Self::setup_linked_button(&marked_button, &unread_button, &all_button, HeaderSelection::Marked);
        Self::setup_update_button(&update_button, &update_stack);

        Self::setup_menu_button(&menu_button);
        Self::setup_mode_button(&mode_button);
        Self::setup_more_actions_button(&more_actions_button);

        ContentHeader {
            update_stack,
            update_button,
            //search_button,
            all_button,
            unread_button,
            marked_button,
            more_actions_button,
            mode_switch_button_label,
        }
    }

    pub fn finish_sync(&self) {
        self.update_button.set_sensitive(true);
        self.update_stack.set_visible_child_name("icon");
    }

    pub fn is_search_focused(&self) -> bool {
        // FIXME
        false
    }

    pub fn focus_search(&self) {
        // FIXME
    }

    pub fn select_all_button(&self) {
        self.all_button.set_active(true);
        self.unread_button.set_active(false);
        self.marked_button.set_active(false);
        self.mode_switch_button_label.set_label("All");
    }

    pub fn select_unread_button(&self) {
        self.unread_button.set_active(true);
        self.all_button.set_active(false);
        self.marked_button.set_active(false);
        self.mode_switch_button_label.set_label("Unread");
    }

    pub fn select_marked_button(&self) {
        self.marked_button.set_active(true);
        self.all_button.set_active(false);
        self.unread_button.set_active(false);
        self.mode_switch_button_label.set_label("Starred");
    }

    fn setup_linked_button(
        button: &ToggleButton,
        other_button_1: &ToggleButton,
        other_button_2: &ToggleButton,
        mode: HeaderSelection,
    ) {
        let other_button_1_1 = other_button_1.clone();
        let other_button_2_1 = other_button_2.clone();
        button.connect_button_press_event(move |button, event| {
            if button.get_active()
            || event.get_button() != 1 {
                return Inhibit(true)
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(true),
            }
            other_button_1_1.set_active(false);
            other_button_2_1.set_active(false);
            Inhibit(false)
        });

        button.connect_toggled(move |button| {
            if !button.get_active() {
                // ignore deactivating toggle-button
                return;
            }
            
            if let Ok(main_window) = GtkUtil::get_main_window(button) {
                if let Some(action) = main_window.lookup_action("headerbar-selection") {
                    if let Ok(json) = serde_json::to_string(&mode) {
                        let json = Variant::from(&json);
                        action.activate(Some(&json));
                    }
                }
            }
        });
    }

    fn setup_update_button(button: &gtk::Button, stack: &gtk::Stack) {
        let stack = stack.clone();
        button.connect_clicked(move |button| {
            button.set_sensitive(false);
            stack.set_visible_child_name("spinner");

            if let Ok(main_window) = GtkUtil::get_main_window(button) {
                if let Some(action) = main_window.lookup_action("sync") {
                    action.activate(None);
                }
            }
        });
    }

    fn setup_menu_button(button: &MenuButton) {
        let about_model = Menu::new();
        about_model.append("Shortcuts", "win.shortcuts");
        about_model.append("About", "win.about");

        let main_model = Menu::new();
        main_model.append("Export OPML", "win.export");
        main_model.append("Settings", "win.settings");
        main_model.append_section("", &about_model);
        
        button.set_menu_model(&main_model);
    }

    fn setup_mode_button(button: &MenuButton) {
        let model = Menu::new();
        if let Ok(json) = serde_json::to_string(&HeaderSelection::All) {
            let variant = Variant::from(&json);
            let all_item = MenuItem::new("All", None);
            all_item.set_action_and_target_value("win.headerbar-selection", &variant);
            model.append_item(&all_item);
        }

        if let Ok(json) = serde_json::to_string(&HeaderSelection::Unread) {
            let variant = Variant::from(&json);
            let unread_item = MenuItem::new("Unread", None);
            unread_item.set_action_and_target_value("win.headerbar-selection", &variant);
            model.append_item(&unread_item);
        }

        if let Ok(json) = serde_json::to_string(&HeaderSelection::Marked) {
            let variant = Variant::from(&json);
            let marked_item = MenuItem::new("Starred", None);
            marked_item.set_action_and_target_value("win.headerbar-selection", &variant);
            model.append_item(&marked_item);
        }
        
        button.set_menu_model(&model);
    }

    fn setup_more_actions_button(button: &MenuButton) {
        let model = Menu::new();
        model.append("Export Article", "win.export-article");
        model.append("Close Article", "win.close-article");
        button.set_menu_model(&model);
        button.set_sensitive(false);
    }

    pub fn set_article_header_sensitive(&self, sensitive: bool) {
        self.more_actions_button.set_sensitive(sensitive);
    }
}
