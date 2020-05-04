use crate::util::BuilderHelper;
use glib::clone;
use gtk::{Box, Button, ButtonExt, HeaderBar, MenuButton, ToggleButton, WidgetExt};
use libhandy::{Leaflet, LeafletExt};
use log::debug;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ResponsiveLayout {
    pub state: Arc<RwLock<ResponsiveState>>,
    pub left_button: Button,
    pub search_button: ToggleButton,
    pub right_button: Button,
    pub major_leaflet: Leaflet,
    pub minor_leaflet: Leaflet,
    pub header_leaflet: Leaflet,
    pub sidebar_box: Box,
    pub article_list_box: Box,
    pub article_view_box: Box,
    pub left_header: HeaderBar,
    pub right_header: HeaderBar,
    pub mode_switch_box: Box,
    pub mode_switch_button: MenuButton,
    pub mark_all_read_button: Button,
}

impl ResponsiveLayout {
    pub fn new(builder: &BuilderHelper) -> Arc<ResponsiveLayout> {
        let state = Arc::new(RwLock::new(ResponsiveState::new()));

        let minor_leaflet = builder.get::<Leaflet>("minor_leaflet");
        let major_leaflet = builder.get::<Leaflet>("major_leaflet");
        let left_button = builder.get::<Button>("left_back_button");
        let search_button = builder.get::<ToggleButton>("search_button");
        let right_button = builder.get::<Button>("right_back_button");
        let sidebar_box = builder.get::<Box>("feedlist_box");
        let article_list_box = builder.get::<Box>("articlelist_box");
        let article_view_box = builder.get::<Box>("articleview_box");
        let header_leaflet = builder.get::<Leaflet>("header_leaflet");
        let left_header = builder.get::<HeaderBar>("left_headerbar");
        let right_header = builder.get::<HeaderBar>("right_headerbar");
        let mode_switch_box = builder.get::<Box>("mode_switch_box");
        let mode_switch_button = builder.get::<MenuButton>("mode_switch_button");
        let mark_all_read_button = builder.get::<Button>("mark_all_button");
        let layout = ResponsiveLayout {
            state,
            left_button,
            search_button,
            right_button,
            major_leaflet,
            minor_leaflet,
            header_leaflet,
            sidebar_box,
            article_list_box,
            article_view_box,
            left_header,
            right_header,
            mode_switch_box,
            mode_switch_button,
            mark_all_read_button,
        };
        let layout = Arc::new(layout);
        Self::setup_signals(&layout);
        layout
    }

    fn setup_signals(layout: &Arc<ResponsiveLayout>) {
        let major_leaflet = layout.major_leaflet.clone();
        let minor_leaflet = layout.minor_leaflet.clone();

        layout.major_leaflet.connect_property_folded_notify(clone!(
            @weak layout => @default-panic, move |_leaflet|
        {
            if minor_leaflet.get_property_folded() {
                debug!("Widget: Minor Leaflet folded");
                layout.state.write().minor_leaflet_folded = true;
                Self::process_state_change(&layout);
            }
            debug!("Widget: Major Leaflet folded");
            layout.state.write().major_leaflet_folded = true;
            Self::process_state_change(&layout);
        }));

        layout.minor_leaflet.connect_property_folded_notify(clone!(
            @weak layout => @default-panic, move |_leaflet|
        {
            if !major_leaflet.get_property_folded() {
                return;
            }
            layout.state.write().minor_leaflet_folded = true;
            Self::process_state_change(&layout);
        }));

        layout
            .left_button
            .connect_clicked(clone!(@weak layout => @default-panic, move |_button| {
                layout.state.write().left_button_clicked = true;
                Self::process_state_change(&layout);
            }));

        layout
            .right_button
            .connect_clicked(clone!(@weak layout => @default-panic, move |_button| {
                layout.state.write().right_button_clicked = true;
                Self::process_state_change(&layout);
            }));
    }

    pub fn process_state_change(&self) {
        if self.state.read().major_leaflet_folded {
            // article view (dis)appeared
            if !self.major_leaflet.get_property_folded() {
                self.right_button.set_visible(false);
                self.major_leaflet.set_visible_child(&self.minor_leaflet);
                self.header_leaflet.set_visible_child(&self.left_header);
                self.mode_switch_box.set_visible(true);
                self.mode_switch_button.set_visible(false);
            } else {
                self.mode_switch_box.set_visible(false);
                self.mode_switch_button.set_visible(true);
            }

            self.state.write().major_leaflet_folded = false;
            return;
        }

        if self.state.read().minor_leaflet_folded {
            // article list (dis)appeared
            log::info!("minor leaflet folded");
            if !self.minor_leaflet.get_property_folded() {
                self.left_button.set_visible(false);
                self.search_button.set_visible(true);
                self.mode_switch_button.set_visible(true);
                self.minor_leaflet.set_visible_child(&self.sidebar_box);
                self.mark_all_read_button.set_visible(true);
            } else {
                self.search_button.set_visible(false);
                self.mode_switch_button.set_visible(false);
                self.mark_all_read_button.set_visible(false);
            }

            self.state.write().minor_leaflet_folded = false;
            return;
        }

        if self.state.read().left_button_clicked {
            // left back
            self.minor_leaflet.set_visible_child(&self.sidebar_box);
            self.left_button.set_visible(false);
            self.search_button.set_visible(false);
            self.mode_switch_button.set_visible(false);
            self.mark_all_read_button.set_visible(false);

            self.state.write().left_button_clicked = false;
            return;
        }

        if self.state.read().right_button_clicked {
            // right back
            //self.minor_leaflet.set_visible_child(&self.article_list_box);
            self.major_leaflet.set_visible_child(&self.minor_leaflet);
            self.header_leaflet.set_visible_child(&self.left_header);
            self.right_button.set_visible(false);

            self.state.write().right_button_clicked = false;
            return;
        }

        if self.state.read().major_leaflet_selected {
            // article selected
            if self.major_leaflet.get_property_folded() {
                self.major_leaflet.set_visible_child(&self.article_view_box);
                self.header_leaflet.set_visible_child(&self.right_header);
                self.right_button.set_visible(true);
            }

            self.state.write().major_leaflet_selected = false;
            return;
        }

        if self.state.read().minor_leaflet_selected {
            // sidebar selected
            if self.minor_leaflet.get_property_folded() {
                self.minor_leaflet.set_visible_child(&self.article_list_box);
                self.left_button.set_visible(true);
                self.search_button.set_visible(true);
                self.mode_switch_button.set_visible(true);
                self.mark_all_read_button.set_visible(true);
            }

            self.state.write().minor_leaflet_selected = false;
            return;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResponsiveState {
    pub left_button_clicked: bool,
    pub right_button_clicked: bool,
    pub major_leaflet_selected: bool,
    pub major_leaflet_folded: bool,
    pub minor_leaflet_selected: bool,
    pub minor_leaflet_folded: bool,
}

impl ResponsiveState {
    pub fn new() -> Self {
        ResponsiveState {
            left_button_clicked: false,
            right_button_clicked: false,
            major_leaflet_selected: false,
            major_leaflet_folded: false,
            minor_leaflet_selected: false,
            minor_leaflet_folded: false,
        }
    }
}
