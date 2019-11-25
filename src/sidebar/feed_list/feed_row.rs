use crate::gtk_handle;
use crate::sidebar::feed_list::models::FeedListFeedModel;
use crate::undo_bar::UndoActionModel;
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
use cairo::{self, Format, ImageSurface};
use gdk::{DragAction, EventType, ModifierType};
use gio::{Menu, MenuItem};
use glib::{source::SourceId, translate::FromGlib, translate::ToGlib, Source, Variant};
use gtk::{
    self, Box, ContainerExt, Continue, DragContextExtManual, EventBox, Image, ImageExt, Inhibit, Label, LabelExt,
    ListBoxRow, ListBoxRowExt, Popover, PopoverExt, PositionType, Revealer, RevealerExt, StateFlags, StyleContextExt,
    TargetEntry, TargetFlags, WidgetExt, WidgetExtManual,
};
use news_flash::models::{CategoryID, FavIcon, FeedID};
use std::cell::RefCell;
use std::rc::Rc;
use std::str;

#[derive(Clone, Debug)]
pub struct FeedRow {
    pub id: FeedID,
    widget: ListBoxRow,
    item_count: Label,
    item_count_event: EventBox,
    title: Label,
    revealer: Revealer,
    hide_timeout: GtkHandle<Option<u32>>,
    favicon: Image,
}

impl FeedRow {
    pub fn new(model: &FeedListFeedModel, visible: bool) -> GtkHandle<FeedRow> {
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
            widget: Self::create_row(&revealer, &model.id, &model.parent_id, &title_label),
            item_count: item_count_label,
            title: title_label,
            revealer,
            hide_timeout: gtk_handle!(None),
            item_count_event,
            favicon,
        };
        feed.update_item_count(model.item_count);
        feed.update_title(&model.label);
        feed.update_favicon(&model.icon);
        if !visible {
            feed.collapse();
        }
        gtk_handle!(feed)
    }

    fn create_row(widget: &gtk::Revealer, id: &FeedID, parent_id: &CategoryID, label: &Label) -> ListBoxRow {
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

            let variant = Variant::from(feed_id.to_str());
            let rename_feed_item = MenuItem::new(Some("Rename"), None);
            rename_feed_item.set_action_and_target_value(Some("rename-feed"), Some(&variant));
            model.append_item(&rename_feed_item);

            let label = match label.get_text() {
                Some(label) => label.as_str().to_owned(),
                None => "".to_owned(),
            };
            let remove_action = UndoActionModel::DeleteFeed((feed_id.clone(), label));
            if let Ok(json) = serde_json::to_string(&remove_action) {
                let variant = Variant::from(json);
                let delete_feed_item = MenuItem::new(Some("Delete"), None);
                delete_feed_item.set_action_and_target_value(Some("enqueue-undoable-action"), Some(&variant));
                model.append_item(&delete_feed_item);
            }

            let popover = Popover::new(Some(row));
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

    pub fn update_favicon(&self, icon: &Option<FavIcon>) {
        if let Some(icon) = icon {
            if let Some(data) = &icon.data {
                let scale = GtkUtil::get_scale(&self.widget());
                if let Ok(surface) = GtkUtil::create_surface_from_bytes(data, 16, 16, scale) {
                    self.favicon.set_from_surface(Some(&surface));
                }
            }
        }
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
                *hide_timeout.borrow_mut() = None;
                Continue(false)
            });
            *self.hide_timeout.borrow_mut() = Some(source_id.to_glib());
        }
    }

    pub fn expand(&self) {
        // clear out timeout to fully hide row
        {
            if let Some(source_id) = *self.hide_timeout.borrow() {
                if Source::remove(SourceId::from_glib(source_id)).is_ok() {
                    // log something
                };
                // log something
            }
            *self.hide_timeout.borrow_mut() = None;
        }

        self.widget.set_visible(true);
        self.revealer.set_reveal_child(true);
        self.revealer.get_style_context().remove_class("hidden");
        self.widget.set_selectable(true);
    }
}
