use super::models::{ArticleListArticleModel, ArticleListModel, MarkUpdate, ReadUpdate};
use crate::app::Action;
use crate::gtk_handle;
use crate::util::{BuilderHelper, DateUtil, GtkHandle, GtkUtil, Util};
use gdk::{EventType, NotifyType};
use glib::Sender;
use gtk::{
    ContainerExt, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, ListBoxRow, ListBoxRowExt, Stack, StackExt,
    StyleContextExt, WidgetExt,
};
use news_flash::models::{ArticleID, Marked, Read};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ArticleRow {
    widget: ListBoxRow,
    marked_handle: GtkHandle<Marked>,
    read_handle: GtkHandle<Read>,
    marked_stack: Stack,
    unread_stack: Stack,
    title_label: Label,
    row_hovered: GtkHandle<bool>,
}

impl ArticleRow {
    pub fn new(
        article: &ArticleListArticleModel,
        list_model: &GtkHandle<ArticleListModel>,
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
        summary_label.set_text(&article.summary);
        feed_label.set_text(&article.feed_title);
        date_label.set_text(&DateUtil::format(&article.date));

        if let Some(icon) = &article.favicon {
            if let Some(data) = &icon.data {
                if let Ok(surface) = GtkUtil::create_surface_from_bytes(data, 16, 16, scale) {
                    favicon.set_from_surface(Some(&surface));
                }
            }
        }

        let read_handle = gtk_handle!(article.read);
        let marked_handle = gtk_handle!(article.marked);
        let row_hovered = gtk_handle!(false);

        Self::setup_row_eventbox(
            &article_eventbox,
            &read_handle,
            &marked_handle,
            &unread_stack,
            &marked_stack,
            &title_label,
            &row_hovered,
        );
        Self::setup_unread_eventbox(
            &sender,
            &unread_eventbox,
            &read_handle,
            &unread_stack,
            &title_label,
            &article.id,
            list_model,
        );
        Self::setup_marked_eventbox(
            &sender,
            &marked_eventbox,
            &marked_handle,
            &marked_stack,
            &article.id,
            list_model,
        );

        ArticleRow {
            widget: row,
            marked_handle,
            read_handle,
            marked_stack,
            unread_stack,
            title_label,
            row_hovered,
        }
    }

    pub fn widget(&self) -> ListBoxRow {
        self.widget.clone()
    }

    pub fn update_marked(&mut self, marked: Marked) {
        Self::update_marked_stack(&self.marked_stack, marked);
        *self.marked_handle.borrow_mut() = marked;
    }

    pub fn update_unread(&mut self, unread: Read) {
        Self::update_title_label(&self.title_label, unread);
        Self::update_unread_stack(&self.unread_stack, unread, *self.row_hovered.borrow());
        *self.read_handle.borrow_mut() = unread;
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
        eventbox: &EventBox,
        read: &GtkHandle<Read>,
        unread_stack: &Stack,
        title_label: &Label,
        article_id: &ArticleID,
        list_model: &GtkHandle<ArticleListModel>,
    ) {
        let read_1 = read.clone();
        let stack_1 = unread_stack.clone();
        let title_label = title_label.clone();
        eventbox.connect_enter_notify_event(move |_widget, _event| {
            match *read_1.borrow() {
                Read::Unread => stack_1.set_visible_child_name("read"),
                Read::Read => stack_1.set_visible_child_name("unread"),
            }
            Inhibit(false)
        });
        let read_2 = read.clone();
        let stack_2 = unread_stack.clone();
        eventbox.connect_leave_notify_event(move |_widget, _event| {
            match *read_2.borrow() {
                Read::Unread => stack_2.set_visible_child_name("unread"),
                Read::Read => stack_2.set_visible_child_name("read"),
            }
            Inhibit(false)
        });
        let read_3 = read.clone();
        let article_id = article_id.clone();
        let list_model = list_model.clone();
        let sender = sender.clone();
        eventbox.connect_button_press_event(move |_widget, event| {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false);
                }
                _ => {}
            }
            let read = *read_3.borrow();
            match read {
                Read::Read => *read_3.borrow_mut() = Read::Unread,
                Read::Unread => *read_3.borrow_mut() = Read::Read,
            }
            let read = *read_3.borrow();
            Self::update_title_label(&title_label, read);
            list_model.borrow_mut().set_read(&article_id, read);
            let update = ReadUpdate {
                article_id: article_id.clone(),
                read,
            };
            Util::send(&sender, Action::MarkArticleRead(update));
            Inhibit(true)
        });
    }

    fn setup_marked_eventbox(
        sender: &Sender<Action>,
        eventbox: &EventBox,
        marked: &GtkHandle<Marked>,
        marked_stack: &Stack,
        article_id: &ArticleID,
        list_model: &GtkHandle<ArticleListModel>,
    ) {
        let marked_1 = marked.clone();
        let stack_1 = marked_stack.clone();
        eventbox.connect_enter_notify_event(move |_widget, _event| {
            match *marked_1.borrow() {
                Marked::Marked => stack_1.set_visible_child_name("unmarked"),
                Marked::Unmarked => stack_1.set_visible_child_name("marked"),
            }
            Inhibit(false)
        });
        let marked_2 = marked.clone();
        let stack_2 = marked_stack.clone();
        eventbox.connect_leave_notify_event(move |_widget, _event| {
            match *marked_2.borrow() {
                Marked::Marked => stack_2.set_visible_child_name("marked"),
                Marked::Unmarked => stack_2.set_visible_child_name("unmarked"),
            }
            Inhibit(false)
        });
        let marked_3 = marked.clone();
        let article_id = article_id.clone();
        let list_model = list_model.clone();
        let sender = sender.clone();
        eventbox.connect_button_press_event(move |_widget, event| {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false);
                }
                _ => {}
            }
            let marked = *marked_3.borrow();
            match marked {
                Marked::Marked => *marked_3.borrow_mut() = Marked::Unmarked,
                Marked::Unmarked => *marked_3.borrow_mut() = Marked::Marked,
            }
            let marked = *marked_3.borrow();
            list_model.borrow_mut().set_marked(&article_id, marked);

            let update = MarkUpdate {
                article_id: article_id.clone(),
                marked,
            };
            Util::send(&sender, Action::MarkArticle(update));
            Inhibit(true)
        });
    }

    fn setup_row_eventbox(
        eventbox: &EventBox,
        read: &GtkHandle<Read>,
        marked: &GtkHandle<Marked>,
        unread_stack: &Stack,
        marked_stack: &Stack,
        title_label: &Label,
        row_hovered: &GtkHandle<bool>,
    ) {
        Self::update_title_label(&title_label, *read.borrow());
        Self::update_unread_stack(&unread_stack, *read.borrow(), *row_hovered.borrow());
        Self::update_marked_stack(&marked_stack, *marked.borrow());

        let read_1 = read.clone();
        let marked_1 = marked.clone();
        let unread_stack_1 = unread_stack.clone();
        let marked_stack_1 = marked_stack.clone();
        let row_hovered_1 = row_hovered.clone();
        eventbox.connect_enter_notify_event(move |_widget, event| {
            if event.get_detail() == NotifyType::Inferior {
                return Inhibit(false);
            }
            *row_hovered_1.borrow_mut() = true;
            match *read_1.borrow() {
                Read::Read => unread_stack_1.set_visible_child_name("read"),
                Read::Unread => unread_stack_1.set_visible_child_name("unread"),
            }
            match *marked_1.borrow() {
                Marked::Marked => marked_stack_1.set_visible_child_name("marked"),
                Marked::Unmarked => marked_stack_1.set_visible_child_name("unmarked"),
            }
            Inhibit(true)
        });

        let read_2 = read.clone();
        let marked_2 = marked.clone();
        let unread_stack_2 = unread_stack.clone();
        let marked_stack_2 = marked_stack.clone();
        let row_hovered_2 = row_hovered.clone();
        eventbox.connect_leave_notify_event(move |_widget, event| {
            if event.get_detail() == NotifyType::Inferior {
                return Inhibit(false);
            }
            *row_hovered_2.borrow_mut() = false;
            match *read_2.borrow() {
                Read::Read => unread_stack_2.set_visible_child_name("empty"),
                Read::Unread => unread_stack_2.set_visible_child_name("unread"),
            }
            match *marked_2.borrow() {
                Marked::Marked => marked_stack_2.set_visible_child_name("marked"),
                Marked::Unmarked => marked_stack_2.set_visible_child_name("empty"),
            }
            Inhibit(true)
        });
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
