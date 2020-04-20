use crate::app::Action;
use crate::main_window_state::MainWindowState;
use crate::sidebar::feed_list::models::FeedListFeedModel;
use crate::undo_bar::UndoActionModel;
use crate::util::{BuilderHelper, GtkUtil, Util};
use cairo::{self, Format, ImageSurface};
use futures::channel::oneshot;
use futures::future::FutureExt;
use gdk::{DragAction, EventType, ModifierType};
use gio::{ActionMapExt, Menu, MenuItem, SimpleAction};
use glib::{
    clone,
    object::Cast,
    source::Continue,
    source::SourceId,
    translate::{FromGlib, ToGlib},
    Sender, Source,
};
use gtk::{
    self, prelude::DragContextExtManual, prelude::WidgetExtManual, BinExt, Box, ContainerExt, EventBox, Image,
    ImageExt, Inhibit, Label, LabelExt, ListBoxRow, ListBoxRowExt, Popover, PopoverExt, PositionType, Revealer,
    RevealerExt, StateFlags, StyleContextExt, TargetEntry, TargetFlags, Widget, WidgetExt,
};
use log::warn;
use news_flash::models::{CategoryID, FavIcon, Feed, FeedID, PluginCapabilities};
use parking_lot::RwLock;
use std::ops::Drop;
use std::str;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct FeedRow {
    pub id: FeedID,
    widget: ListBoxRow,
    item_count: Label,
    item_count_event: EventBox,
    title: Label,
    revealer: Revealer,
    hide_timeout: Arc<RwLock<Option<u32>>>,
    favicon: Image,
    connected_signals: Vec<(u64, Widget)>,
}

impl FeedRow {
    pub fn new(
        model: &FeedListFeedModel,
        state: &Arc<RwLock<MainWindowState>>,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
        visible: bool,
        sender: Sender<Action>,
    ) -> Arc<RwLock<FeedRow>> {
        let builder = BuilderHelper::new("feed");
        let revealer = builder.get::<Revealer>("feed_row");
        let level_margin = builder.get::<Box>("level_margin");
        level_margin.set_margin_start(model.level * 24);

        let title_label = builder.get::<Label>("feed_title");
        let item_count_label = builder.get::<Label>("item_count");
        let item_count_event = builder.get::<EventBox>("item_count_event");
        let favicon = builder.get::<Image>("favicon");

        let mut feed = FeedRow {
            id: model.id.clone(),
            widget: ListBoxRow::new(),
            item_count: item_count_label,
            title: title_label,
            revealer: revealer.clone(),
            hide_timeout: Arc::new(RwLock::new(None)),
            item_count_event,
            favicon,
            connected_signals: Vec::new(),
        };
        feed.connected_signals = Self::setup_row(
            &feed.widget,
            &sender,
            state,
            features,
            &revealer,
            &model.id,
            &model.parent_id,
            model.label.clone(),
        );
        feed.update_item_count(model.item_count);
        feed.update_title(&model.label);
        feed.update_favicon(&model.news_flash_model, &sender);
        if !visible {
            feed.collapse();
        }
        Arc::new(RwLock::new(feed))
    }

    fn setup_row(
        row: &ListBoxRow,
        sender: &Sender<Action>,
        window_state: &Arc<RwLock<MainWindowState>>,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
        revealer: &gtk::Revealer,
        id: &FeedID,
        parent_id: &CategoryID,
        label: String,
    ) -> Vec<(u64, Widget)> {
        let mut vec = Vec::new();

        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(revealer);
        row.get_style_context().remove_class("activatable");

        let mut support_mutation = false;
        if let Some(features) = features.read().as_ref() {
            support_mutation = features.contains(PluginCapabilities::ADD_REMOVE_FEEDS)
                && features.contains(PluginCapabilities::MODIFY_CATEGORIES);
        }

        if support_mutation {
            let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
            revealer.drag_source_set(ModifierType::BUTTON1_MASK, &[entry], DragAction::MOVE);
            revealer.drag_source_add_text_targets();

            vec.push((
                revealer
                    .connect_drag_data_get(clone!(
                        @strong parent_id,
                        @strong id as feed_id => @default-panic, move |_widget, _ctx, selection_data, _info, _time|
                    {
                        if let Ok(feed_id_json) = serde_json::to_string(&feed_id.clone()) {
                            if let Ok(category_id_json) = serde_json::to_string(&parent_id.clone()) {
                                let mut data = String::from("FeedID ");
                                data.push_str(&feed_id_json);
                                data.push_str(";");
                                data.push_str(&category_id_json);
                                selection_data.set_text(&data);
                            }
                        }
                    }))
                    .to_glib(),
                revealer.clone().upcast::<Widget>(),
            ));

            vec.push((
                revealer
                    .connect_drag_begin(clone!(@weak row => @default-panic, move |_widget, drag_context| {
                        let alloc = row.get_allocation();
                        let surface = ImageSurface::create(Format::ARgb32, alloc.width, alloc.height)
                            .expect("Failed to create Cairo ImageSurface.");
                        let cairo_context = cairo::Context::new(&surface);
                        let style_context = row.get_style_context();
                        style_context.add_class("drag-icon");
                        row.draw(&cairo_context);
                        style_context.remove_class("drag-icon");
                        drag_context.drag_set_icon_surface(&surface);
                    }))
                    .to_glib(),
                revealer.clone().upcast::<Widget>(),
            ));

            vec.push((row.connect_button_press_event(clone!(
                @strong id as feed_id,
                @strong label,
                @weak window_state,
                @strong sender => @default-panic, move |row, event|
            {
                if event.get_button() != 3 {
                    return Inhibit(false);
                }

                match event.get_event_type() {
                    EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                        return Inhibit(false)
                    }
                    _ => {}
                }

                if window_state.read().get_offline() {
                    return Inhibit(false);
                }

                let model = Menu::new();

                let rename_feed_dialog_action = SimpleAction::new(&format!("rename-feed-{}-dialog", feed_id), None);
                rename_feed_dialog_action.connect_activate(clone!(@weak row, @strong feed_id, @strong sender => @default-panic, move |_action, _parameter| {
                    Util::send(&sender, Action::RenameFeedDialog(feed_id.clone()));

                    if let Ok(main_window) = GtkUtil::get_main_window(&row) {
                        main_window.remove_action(&format!("rename-feed-{}-dialog", feed_id));
                    }
                }));

                let rename_feed_item = MenuItem::new(Some("Rename"), None);
                rename_feed_item.set_action_and_target_value(Some(&format!("rename-feed-{}-dialog", feed_id)), None);
                model.append_item(&rename_feed_item);

                let delete_feed_item = MenuItem::new(Some("Delete"), None);
                let delete_feed_action = SimpleAction::new(&format!("enqueue-delete-feed-{}", feed_id), None);
                delete_feed_action.connect_activate(clone!(
                    @weak row,
                    @strong label,
                    @strong feed_id,
                    @strong sender => @default-panic, move |_action, _parameter|
                {
                    let remove_action = UndoActionModel::DeleteFeed((feed_id.clone(), label.clone()));
                    Util::send(&sender, Action::UndoableAction(remove_action));

                    if let Ok(main_window) = GtkUtil::get_main_window(&row) {
                        main_window.remove_action(&format!("enqueue-delete-feed-{}", feed_id));
                    }
                }));

                if let Ok(main_window) = GtkUtil::get_main_window(row) {
                    main_window.add_action(&delete_feed_action);
                    main_window.add_action(&rename_feed_dialog_action);
                }
                delete_feed_item.set_action_and_target_value(Some(&format!("enqueue-delete-feed-{}", feed_id)), None);
                model.append_item(&delete_feed_item);

                let popover = Popover::new(Some(row));
                popover.set_position(PositionType::Bottom);
                popover.bind_model(Some(&model), Some("win"));
                popover.show();
                popover.connect_closed(clone!(@weak row => @default-panic, move |_popover| {
                    row.unset_state_flags(StateFlags::PRELIGHT);
                }));
                row.set_state_flags(StateFlags::PRELIGHT, false);

                Inhibit(true)
            })).to_glib(), row.clone().upcast::<Widget>()));
        }

        vec
    }

    pub fn widget(&self) -> ListBoxRow {
        self.widget.clone()
    }

    pub fn update_item_count(&self, count: i64) {
        if count > 0 {
            self.item_count.set_label(&count.to_string());
            self.item_count_event.set_visible(true);
        } else {
            self.item_count_event.set_visible(false);
        }
    }

    pub fn update_favicon(&self, feed: &Option<Feed>, global_sender: &Sender<Action>) {
        let (sender, receiver) = oneshot::channel::<Option<FavIcon>>();
        if let Some(feed) = feed {
            Util::send(global_sender, Action::LoadFavIcon((feed.clone(), sender)));
        }

        let scale = GtkUtil::get_scale(&self.widget);
        let glib_future = receiver.map(
            clone!(@weak self.favicon as favicon => @default-panic, move |res| match res {
                Ok(Some(icon)) => {
                    if let Some(data) = &icon.data {
                        if let Ok(surface) = GtkUtil::create_surface_from_bytes(data, 16, 16, scale) {
                            favicon.set_from_surface(Some(&surface));
                        }
                    }
                }
                Ok(None) => {
                    warn!("Favicon does not contain image data.");
                }
                Err(_) => warn!("Receiving favicon failed."),
            }),
        );

        Util::glib_spawn_future(glib_future);
    }

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }

    pub fn collapse(&mut self) {
        self.revealer.set_reveal_child(false);
        self.revealer.get_style_context().add_class("hidden");
        self.widget.set_selectable(false);

        // hide row after animation finished
        {
            // add new timeout
            let source_id = gtk::timeout_add(
                250,
                clone!(
                    @weak self.hide_timeout as hide_timeout,
                    @weak self.widget as widget => @default-panic, move ||
                {
                    widget.set_visible(false);
                    hide_timeout.write().take();
                    Continue(false)
                }),
            );
            self.hide_timeout.write().replace(source_id.to_glib());
        }
    }

    pub fn expand(&self) {
        // clear out timeout to fully hide row
        {
            if let Some(source_id) = *self.hide_timeout.read() {
                if Source::remove(SourceId::from_glib(source_id)).is_ok() {
                    // log something
                };
                // log something
            }
            self.hide_timeout.write().take();
        }

        self.widget.set_visible(true);
        self.revealer.set_reveal_child(true);
        self.revealer.get_style_context().remove_class("hidden");
        self.widget.set_selectable(true);
    }

    pub fn disable_dnd(&self) {
        if let Some(widget) = self.widget.get_child() {
            widget.drag_source_unset();
        }
    }

    pub fn enable_dnd(&self) {
        if let Some(widget) = self.widget.get_child() {
            let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
            widget.drag_source_set(ModifierType::BUTTON1_MASK, &[entry], DragAction::MOVE);
        }
    }
}

impl Drop for FeedRow {
    fn drop(&mut self) {
        for (signal_id, widget) in &self.connected_signals {
            GtkUtil::disconnect_signal(Some(*signal_id), widget);
        }
    }
}
