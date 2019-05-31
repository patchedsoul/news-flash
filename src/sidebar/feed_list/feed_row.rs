use crate::gtk_handle;
use crate::sidebar::feed_list::models::FeedListFeedModel;
use crate::util::BuilderHelper;
use crate::util::GtkHandle;
use crate::util::GtkUtil;
use cairo::{self, Format, ImageSurface};
use gdk::{DragAction, ModifierType};
use glib::{source::SourceId, translate::FromGlib, translate::ToGlib, Source};
use gtk::{
    self, Box, ContainerExt, Continue, DragContextExtManual, EventBox, Image, ImageExt, Label, LabelExt, ListBoxRow,
    ListBoxRowExt, Revealer, RevealerExt, StyleContextExt, TargetEntry, TargetFlags, WidgetExt, WidgetExtManual,
};
use news_flash::models::{FavIcon, FeedID};
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
            widget: Self::create_row(&revealer, &model.id),
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

    fn create_row(widget: &gtk::Revealer, id: &FeedID) -> ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context();
        context.remove_class("activatable");
        let row_2nd_handle = row.clone();
        let id = id.clone();

        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        widget.drag_source_set(ModifierType::BUTTON1_MASK, &[entry], DragAction::MOVE);
        widget.drag_source_add_text_targets();
        widget.connect_drag_data_get(move |_widget, _ctx, selection_data, _info, _time| {
            if let Ok(json) = serde_json::to_string(&id.clone()) {
                let mut data = String::from("FeedID ");
                data.push_str(&json);
                selection_data.set_text(&data);
            }
        });
        widget.connect_drag_begin(move |_widget, drag_context| {
            let alloc = row.get_allocation();
            let surface = ImageSurface::create(Format::ARgb32, alloc.width, alloc.height).unwrap();
            let cairo_context = cairo::Context::new(&surface);
            let style_context = row.get_style_context();
            style_context.add_class("drag-icon");
            row.draw(&cairo_context);
            style_context.remove_class("drag-icon");
            drag_context.drag_set_icon_surface(&surface);
        });

        row_2nd_handle
    }

    pub fn widget(&self) -> ListBoxRow {
        self.widget.clone()
    }

    pub fn update_item_count(&self, count: i32) {
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
                let scale = self.widget.get_style_context().get_scale();
                let surface = GtkUtil::create_surface_from_bytes(data, 16, 16, scale).unwrap();
                self.favicon.set_from_surface(&surface);
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
