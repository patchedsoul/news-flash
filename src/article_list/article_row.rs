use super::models::{ArticleListArticleModel, ArticleListModel, MarkUpdate, ReadUpdate};
use crate::app::Action;
use crate::main_window_state::MainWindowState;
use crate::util::{BuilderHelper, DateUtil, GtkUtil, Util};
use futures::channel::oneshot;
use futures::future::FutureExt;
use gdk::{EventType, NotifyType};
use glib::{clone, object::Cast, translate::ToGlib, Sender};
use gtk::{
    ContainerExt, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, ListBoxRow, ListBoxRowExt, Stack, StackExt,
    StyleContextExt, Widget, WidgetExt,
};
use log::warn;
use news_flash::models::{ArticleID, FavIcon, Marked, Read};
use parking_lot::RwLock;
use std::ops::Drop;
use std::sync::Arc;

pub struct ArticleRow {
    widget: ListBoxRow,
    marked_handle: Arc<RwLock<Marked>>,
    read_handle: Arc<RwLock<Read>>,
    marked_stack: Stack,
    unread_stack: Stack,
    title_label: Label,
    row_hovered: Arc<RwLock<bool>>,
    connected_signals: Vec<(usize, Widget)>,
}

impl ArticleRow {
    pub fn new(
        article: &ArticleListArticleModel,
        list_model: &Arc<RwLock<ArticleListModel>>,
        state: &Arc<RwLock<MainWindowState>>,
        sender: Sender<Action>,
    ) -> Self {
        let builder = BuilderHelper::new("article");

        let favicon = builder.get::<Image>("favicon");
        let article_eventbox = builder.get::<EventBox>("article_eventbox");
        let unread_eventbox = builder.get::<EventBox>("unread_eventbox");
        let marked_eventbox = builder.get::<EventBox>("marked_eventbox");
        let unread_stack = builder.get::<Stack>("unread_stack");
        let marked_stack = builder.get::<Stack>("marked_stack");
        let title_label = builder.get::<Label>("title_label");
        let summary_label = builder.get::<Label>("summary_label");
        let feed_label = builder.get::<Label>("feed_label");
        let date_label = builder.get::<Label>("date_label");

        let row = Self::create_row(&article_eventbox);

        let scale = GtkUtil::get_scale(&favicon);

        let marked = builder.get::<Image>("marked");
        let surface = GtkUtil::create_surface_from_icon_name("marked", 16, scale);
        marked.set_from_surface(Some(&surface));

        let unmarked = builder.get::<Image>("unmarked");
        let surface = GtkUtil::create_surface_from_icon_name("unmarked", 16, scale);
        unmarked.set_from_surface(Some(&surface));

        let read = builder.get::<Image>("read");
        let surface = GtkUtil::create_surface_from_icon_name("read", 16, scale);
        read.set_from_surface(Some(&surface));

        let unread = builder.get::<Image>("unread");
        let surface = GtkUtil::create_surface_from_icon_name("unread", 16, scale);
        unread.set_from_surface(Some(&surface));

        title_label.set_text(&article.title);
        title_label.set_tooltip_text(Some(&article.title));
        summary_label.set_text(&article.summary);
        feed_label.set_text(&article.feed_title);
        date_label.set_text(&DateUtil::format(&article.date));

        let (oneshot_sender, receiver) = oneshot::channel::<Option<FavIcon>>();
        Util::send(
            &sender,
            Action::LoadFavIcon((article.news_flash_feed.clone(), oneshot_sender)),
        );
        let glib_future = receiver.map(move |res| match res {
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
        });
        Util::glib_spawn_future(glib_future);

        let read_handle = Arc::new(RwLock::new(article.read));
        let marked_handle = Arc::new(RwLock::new(article.marked));
        let row_hovered = Arc::new(RwLock::new(false));

        let mut connected_signals = Vec::new();

        connected_signals.append(&mut Self::setup_row_eventbox(
            &article_eventbox,
            &read_handle,
            &marked_handle,
            &unread_stack,
            &marked_stack,
            &title_label,
            &row_hovered,
        ));
        connected_signals.append(&mut Self::setup_unread_eventbox(
            &sender,
            state,
            &unread_eventbox,
            &read_handle,
            &unread_stack,
            &title_label,
            &article.id,
            list_model,
        ));
        connected_signals.append(&mut Self::setup_marked_eventbox(
            &sender,
            state,
            &marked_eventbox,
            &marked_handle,
            &marked_stack,
            &article.id,
            list_model,
        ));

        ArticleRow {
            widget: row,
            marked_handle,
            read_handle,
            marked_stack,
            unread_stack,
            title_label,
            row_hovered,
            connected_signals,
        }
    }

    pub fn widget(&self) -> ListBoxRow {
        self.widget.clone()
    }

    pub fn update_marked(&mut self, marked: Marked) {
        Self::update_marked_stack(&self.marked_stack, marked);
        *self.marked_handle.write() = marked;
    }

    pub fn update_unread(&mut self, unread: Read) {
        Self::update_title_label(&self.title_label, unread);
        Self::update_unread_stack(&self.unread_stack, unread, *self.row_hovered.read());
        *self.read_handle.write() = unread;
    }

    fn create_row(widget: &EventBox) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context();
        context.remove_class("activatable");

        row
    }

    fn setup_unread_eventbox(
        sender: &Sender<Action>,
        state: &Arc<RwLock<MainWindowState>>,
        eventbox: &EventBox,
        read: &Arc<RwLock<Read>>,
        unread_stack: &Stack,
        title_label: &Label,
        article_id: &ArticleID,
        list_model: &Arc<RwLock<ArticleListModel>>,
    ) -> Vec<(usize, Widget)> {
        let mut vec = Vec::new();
        vec.push((
            eventbox
                .connect_enter_notify_event(clone!(
                    @weak unread_stack,
                    @weak state as window_state,
                    @weak read => @default-panic, move |_widget, _event|
                {
                    if !window_state.read().get_offline() {
                        match *read.read() {
                            Read::Unread => unread_stack.set_visible_child_name("read"),
                            Read::Read => unread_stack.set_visible_child_name("unread"),
                        }
                    }
                    Inhibit(false)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec.push((
            eventbox
                .connect_leave_notify_event(clone!(
                    @weak state as window_state,
                    @weak unread_stack,
                    @weak read => @default-panic, move |_widget, _event|
                {
                    if !window_state.read().get_offline() {
                        match *read.read() {
                            Read::Unread => unread_stack.set_visible_child_name("unread"),
                            Read::Read => unread_stack.set_visible_child_name("read"),
                        }
                    }
                    Inhibit(false)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec.push((
            eventbox
                .connect_button_press_event(clone!(
                    @weak state as window_state,
                    @weak list_model,
                    @weak title_label,
                    @weak read,
                    @strong article_id,
                    @strong sender => @default-panic, move |_widget, event|
                {
                    if event.get_button() != 1 {
                        return Inhibit(false);
                    }
                    match event.get_event_type() {
                        EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                            return Inhibit(false);
                        }
                        _ => {}
                    }
                    if window_state.read().get_offline() {
                        return Inhibit(false);
                    }

                    let new_state = read.read().invert();
                    *read.write() = new_state;
                    Self::update_title_label(&title_label, new_state);
                    list_model.write().set_read(&article_id, new_state);
                    let update = ReadUpdate {
                        article_id: article_id.clone(),
                        read: new_state,
                    };
                    Util::send(&sender, Action::MarkArticleRead(update));
                    Inhibit(true)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec
    }

    fn setup_marked_eventbox(
        sender: &Sender<Action>,
        state: &Arc<RwLock<MainWindowState>>,
        eventbox: &EventBox,
        marked: &Arc<RwLock<Marked>>,
        marked_stack: &Stack,
        article_id: &ArticleID,
        list_model: &Arc<RwLock<ArticleListModel>>,
    ) -> Vec<(usize, Widget)> {
        let mut vec = Vec::new();

        vec.push((
            eventbox
                .connect_enter_notify_event(clone!(
                    @weak marked_stack,
                    @weak state as window_state,
                    @weak marked => @default-panic, move |_widget, _event|
                {
                    if !window_state.read().get_offline() {
                        match *marked.read() {
                            Marked::Marked => marked_stack.set_visible_child_name("unmarked"),
                            Marked::Unmarked => marked_stack.set_visible_child_name("marked"),
                        }
                    }
                    Inhibit(false)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec.push((
            eventbox
                .connect_leave_notify_event(clone!(
                    @weak marked_stack,
                    @weak state as window_state,
                    @weak marked => @default-panic, move |_widget, _event|
                {
                    if !window_state.read().get_offline() {
                        match *marked.read() {
                            Marked::Marked => marked_stack.set_visible_child_name("marked"),
                            Marked::Unmarked => marked_stack.set_visible_child_name("unmarked"),
                        }
                    }
                    Inhibit(false)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec.push((
            eventbox
                .connect_button_press_event(clone!(
                    @strong sender,
                    @strong article_id,
                    @weak list_model,
                    @weak state as window_state,
                    @weak marked => @default-panic, move |_widget, event|
                {
                    if event.get_button() != 1 {
                        return Inhibit(false);
                    }
                    match event.get_event_type() {
                        EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                            return Inhibit(false);
                        }
                        _ => {}
                    }
                    if window_state.read().get_offline() {
                        return Inhibit(false);
                    }
                    let new_marked = marked.read().invert();
                    *marked.write() = new_marked;
                    list_model.write().set_marked(&article_id, new_marked);

                    let update = MarkUpdate {
                        article_id: article_id.clone(),
                        marked: new_marked,
                    };
                    Util::send(&sender, Action::MarkArticle(update));
                    Inhibit(true)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec
    }

    fn setup_row_eventbox(
        eventbox: &EventBox,
        read: &Arc<RwLock<Read>>,
        marked: &Arc<RwLock<Marked>>,
        unread_stack: &Stack,
        marked_stack: &Stack,
        title_label: &Label,
        row_hovered: &Arc<RwLock<bool>>,
    ) -> Vec<(usize, Widget)> {
        Self::update_title_label(&title_label, *read.read());
        Self::update_unread_stack(&unread_stack, *read.read(), *row_hovered.read());
        Self::update_marked_stack(&marked_stack, *marked.read());

        let mut vec = Vec::new();

        vec.push((
            eventbox
                .connect_enter_notify_event(clone!(
                    @weak row_hovered,
                    @weak unread_stack,
                    @weak marked_stack,
                    @weak marked,
                    @weak read => @default-panic, move |_widget, event|
                {
                    if event.get_detail() == NotifyType::Inferior {
                        return Inhibit(false);
                    }
                    *row_hovered.write() = true;
                    match *read.read() {
                        Read::Read => unread_stack.set_visible_child_name("read"),
                        Read::Unread => unread_stack.set_visible_child_name("unread"),
                    }
                    match *marked.read() {
                        Marked::Marked => marked_stack.set_visible_child_name("marked"),
                        Marked::Unmarked => marked_stack.set_visible_child_name("unmarked"),
                    }
                    Inhibit(true)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec.push((
            eventbox
                .connect_leave_notify_event(clone!(
                    @weak row_hovered,
                    @weak marked_stack,
                    @weak unread_stack,
                    @weak marked,
                    @weak read => @default-panic, move |_widget, event|
                {
                    if event.get_detail() == NotifyType::Inferior {
                        return Inhibit(false);
                    }
                    *row_hovered.write() = false;
                    match *read.read() {
                        Read::Read => unread_stack.set_visible_child_name("empty"),
                        Read::Unread => unread_stack.set_visible_child_name("unread"),
                    }
                    match *marked.read() {
                        Marked::Marked => marked_stack.set_visible_child_name("marked"),
                        Marked::Unmarked => marked_stack.set_visible_child_name("empty"),
                    }
                    Inhibit(true)
                }))
                .to_glib() as usize,
            eventbox.clone().upcast::<Widget>(),
        ));

        vec
    }

    fn update_title_label(title_label: &Label, read: Read) {
        let context = title_label.get_style_context();
        match read {
            Read::Read => context.remove_class("bold"),
            Read::Unread => context.add_class("bold"),
        }
    }

    fn update_unread_stack(unread_stack: &Stack, read: Read, row_hovered: bool) {
        match read {
            Read::Read => {
                if row_hovered {
                    unread_stack.set_visible_child_name("read")
                } else {
                    unread_stack.set_visible_child_name("empty")
                }
            }
            Read::Unread => unread_stack.set_visible_child_name("unread"),
        }
    }

    fn update_marked_stack(marked_stack: &Stack, marked: Marked) {
        match marked {
            Marked::Unmarked => marked_stack.set_visible_child_name("empty"),
            Marked::Marked => marked_stack.set_visible_child_name("marked"),
        }
    }
}

impl Drop for ArticleRow {
    fn drop(&mut self) {
        for (signal_id, widget) in &self.connected_signals {
            GtkUtil::disconnect_signal(Some(*signal_id), widget);
        }
    }
}
