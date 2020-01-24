use crate::app::Action;
use crate::sidebar::feed_list::models::FeedListFeedModel;
use crate::undo_bar::UndoActionModel;
use crate::util::{BuilderHelper, GtkUtil, Util};
use cairo::{self, Format, ImageSurface};
use futures::channel::oneshot;
use futures::future::FutureExt;
use gdk::{DragAction, EventType, ModifierType};
use gio::{ActionMapExt, Menu, MenuItem, SimpleAction};
use glib::{source::SourceId, translate::FromGlib, translate::ToGlib, Sender, Source};
use gtk::{
    self, Box, ContainerExt, Continue, DragContextExtManual, EventBox, Image, ImageExt, Inhibit, Label, LabelExt,
    ListBoxRow, ListBoxRowExt, Popover, PopoverExt, PositionType, Revealer, RevealerExt, StateFlags, StyleContextExt,
    TargetEntry, TargetFlags, WidgetExt, WidgetExtManual,
};
use news_flash::models::{CategoryID, FavIcon, Feed, FeedID};
use parking_lot::RwLock;
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
}

impl FeedRow {
    pub fn new(model: &FeedListFeedModel, visible: bool, sender: Sender<Action>) -> Arc<RwLock<FeedRow>> {
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
            widget: Self::create_row(&sender, &revealer, &model.id, &model.parent_id, &title_label),
            item_count: item_count_label,
            title: title_label,
            revealer,
            hide_timeout: Arc::new(RwLock::new(None)),
            item_count_event,
            favicon,
        };
        feed.update_item_count(model.item_count);
        feed.update_title(&model.label);
        feed.update_favicon(&model.news_flash_model, &sender);
        if !visible {
            feed.collapse();
        }
        Arc::new(RwLock::new(feed))
    }

    fn create_row(
        sender: &Sender<Action>,
        widget: &gtk::Revealer,
        id: &FeedID,
        parent_id: &CategoryID,
        label: &Label,
    ) -> ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(widget);
        row.get_style_context().remove_class("activatable");

        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        widget.drag_source_set(ModifierType::BUTTON1_MASK, &[entry], DragAction::MOVE);
        widget.drag_source_add_text_targets();
        let feed_id = id.clone();
        let parent_id = parent_id.clone();
        widget.connect_drag_data_get(move |_widget, _ctx, selection_data, _info, _time| {
            if let Ok(feed_id_json) = serde_json::to_string(&feed_id.clone()) {
                if let Ok(category_id_json) = serde_json::to_string(&parent_id.clone()) {
                    let mut data = String::from("FeedID ");
                    data.push_str(&feed_id_json);
                    data.push_str(";");
                    data.push_str(&category_id_json);
                    selection_data.set_text(&data);
                }
            }
        });
        let row_clone = row.clone();
        widget.connect_drag_begin(move |_widget, drag_context| {
            let alloc = row_clone.get_allocation();
            let surface = ImageSurface::create(Format::ARgb32, alloc.width, alloc.height)
                .expect("Failed to create Cairo ImageSurface.");
            let cairo_context = cairo::Context::new(&surface);
            let style_context = row_clone.get_style_context();
            style_context.add_class("drag-icon");
            row_clone.draw(&cairo_context);
            style_context.remove_class("drag-icon");
            drag_context.drag_set_icon_surface(&surface);
        });

        let feed_id = id.clone();
        let label = label.clone();
        let sender = sender.clone();
        row.connect_button_press_event(move |row, event| {
            if event.get_button() != 3 {
                return Inhibit(false);
            }

            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false)
                }
                _ => {}
            }

            let model = Menu::new();
            model.append(Some("Move"), Some("move-feed"));

            let sender_clone = sender.clone();
            let feed_id_clone = feed_id.clone();
            let rename_feed_dialog_action = SimpleAction::new("rename-feed-dialog", None);
            rename_feed_dialog_action.connect_activate(move |_action, _parameter| {
                let feed_id = feed_id_clone.clone();
                Util::send(&sender_clone, Action::RenameFeedDialog(feed_id));
            });

            if let Ok(main_window) = GtkUtil::get_main_window(row) {
                main_window.add_action(&rename_feed_dialog_action);
            }

            let rename_feed_item = MenuItem::new(Some("Rename"), None);
            rename_feed_item.set_action_and_target_value(Some("rename-feed-dialog"), None);
            model.append_item(&rename_feed_item);

            let delete_feed_item = MenuItem::new(Some("Delete"), None);
            let delete_feed_action = SimpleAction::new("enqueue-delete-feed", None);
            let sender = sender.clone();
            let feed_id = feed_id.clone();
            let label = label.clone();
            let row = row.clone();
            delete_feed_action.connect_activate(move |_action, _parameter| {
                let label = match label.get_text() {
                    Some(label) => label.as_str().to_owned(),
                    None => "".to_owned(),
                };
                let remove_action = UndoActionModel::DeleteFeed((feed_id.clone(), label));
                Util::send(&sender, Action::UndoableAction(remove_action));
            });
            if let Ok(main_window) = GtkUtil::get_main_window(&row) {
                main_window.add_action(&delete_feed_action);
            }
            delete_feed_item.set_action_and_target_value(Some("enqueue-delete-feed"), None);
            model.append_item(&delete_feed_item);

            let popover = Popover::new(Some(&row));
            popover.set_position(PositionType::Bottom);
            popover.bind_model(Some(&model), Some("win"));
            popover.show();
            let row_clone = row.clone();
            popover.connect_closed(move |_popover| {
                row_clone.unset_state_flags(StateFlags::PRELIGHT);
            });
            row.set_state_flags(StateFlags::PRELIGHT, false);

            Inhibit(true)
        });

        row
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

        let favicon = self.favicon.clone();
        let scale = GtkUtil::get_scale(&self.widget());
        let glib_future = receiver.map(move |res| {
            if let Some(icon) = res.unwrap() {
                if let Some(data) = &icon.data {
                    if let Ok(surface) = GtkUtil::create_surface_from_bytes(data, 16, 16, scale) {
                        favicon.set_from_surface(Some(&surface));
                    }
                }
            }
        });

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
            let widget = self.widget();
            let hide_timeout = self.hide_timeout.clone();
            let source_id = gtk::timeout_add(250, move || {
                widget.set_visible(false);
                hide_timeout.write().take();
                Continue(false)
            });
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
}
