use gtk::{
    self,
    LabelExt,
    ContainerExt,
    WidgetExt,
    WidgetExtManual,
    StyleContextExt,
    ListBoxRowExt,
    RevealerExt,
    TargetEntry,
    TargetFlags,
    DragContextExtManual,
    ImageExt,
    Continue,
};
use gdk::{
    DragAction,
    ModifierType,
};
use glib::{
    Source,
    translate::ToGlib,
};
use cairo::{
    self,
    ImageSurface,
    Format,
};
use news_flash::models::{
    FeedID,
    FavIcon,
};
use crate::sidebar::feed_list::models::{
    FeedListFeedModel,
};
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use crate::Resources;
use crate::gtk_util::GtkUtil;
use crate::main_window::GtkHandle;

#[derive(Clone, Debug)]
pub struct FeedRow {
    pub id: FeedID,
    widget: gtk::ListBoxRow,
    item_count: gtk::Label,
    item_count_event: gtk::EventBox,
    title: gtk::Label,
    revealer: gtk::Revealer,
    hide_timeout: GtkHandle<Option<u32>>,
    favicon: gtk::Image,
}

impl FeedRow {
    pub fn new(model: &FeedListFeedModel, visible: bool) -> GtkHandle<FeedRow> {
        let ui_data = Resources::get("ui/feed.ui").unwrap();
        let ui_string = str::from_utf8(ui_data.as_ref()).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let revealer : gtk::Revealer = builder.get_object("feed_row").unwrap();
        let level_margin : gtk::Box = builder.get_object("level_margin").unwrap();
        level_margin.set_margin_start(model.level*24);
        
        let title_label : gtk::Label = builder.get_object("feed_title").unwrap();
        let item_count_label : gtk::Label = builder.get_object("item_count").unwrap();
        let item_count_event : gtk::EventBox = builder.get_object("item_count_event").unwrap();
        let favicon : gtk::Image = builder.get_object("favicon").unwrap();

        let mut feed = FeedRow {
            id: model.id.clone(),
            widget: Self::create_row(&revealer, &model.id),
            item_count: item_count_label,
            title: title_label,
            revealer: revealer,
            hide_timeout: Rc::new(RefCell::new(None)),
            item_count_event: item_count_event,
            favicon: favicon,
        };
        feed.update_item_count(model.item_count);
        feed.update_title(&model.label);
        feed.update_favicon(&model.icon);
        if !visible {
            feed.collapse();
        }
        Rc::new(RefCell::new(feed))
    }

    fn create_row(widget: &gtk::Revealer, id: &FeedID) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(false);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context().unwrap();
        context.remove_class("activatable");
        let row_2nd_handle = row.clone();
        let id = id.clone();

        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        widget.drag_source_set(ModifierType::BUTTON1_MASK, &vec![entry], DragAction::MOVE);
        widget.drag_source_add_text_targets();
        widget.connect_drag_data_get(move |_widget, _ctx, selection_data, _info, _time| {
            if let Ok(json) = serde_json::to_string(&id.clone()) {
                let mut data =  String::from("FeedID ");
                data.push_str(&json);
                selection_data.set_text(&data);
            }
        });
        widget.connect_drag_begin(move |_widget, drag_context| {
            let alloc = row.get_allocation();
            let surface = ImageSurface::create(Format::ARgb32, alloc.width, alloc.height).unwrap();
            let cairo_context = cairo::Context::new(&surface);
            let style_context = row.get_style_context().unwrap();
            style_context.add_class("drag-icon");
            row.draw(&cairo_context);
            style_context.remove_class("drag-icon");
            drag_context.drag_set_icon_surface(&surface);
        });
        
        row_2nd_handle
    }
    
    pub fn row(&self) -> gtk::ListBoxRow {
        self.widget.clone()
    }

    pub fn update_item_count(&self, count: i32) {
        if count > 0 {
            self.item_count.set_label(&count.to_string());
            self.item_count_event.set_visible(true);
        }
        else {
            self.item_count_event.set_visible(false);
        }
    }

    pub fn update_favicon(&self, icon: &Option<FavIcon>) {
        if let Some(icon) = icon {
            if let Some(data) = &icon.data {
                let scale = match self.widget.get_style_context() {
                    Some(ctx) => ctx.get_scale(),
                    None => 1,
                };
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
        self.revealer.get_style_context().unwrap().add_class("hidden");
        self.widget.set_selectable(false);

        // hide row after animation finished
        {
            // add new timeout
            let widget = self.row();
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
                Source::remove(source_id);
            }
            *self.hide_timeout.borrow_mut() = None;
        }

        self.widget.set_visible(true);
        self.revealer.set_reveal_child(true);
        self.revealer.get_style_context().unwrap().remove_class("hidden");
        self.widget.set_selectable(true);
    }
}
