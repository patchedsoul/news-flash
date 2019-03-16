use gtk::{
    Builder,
    ListBoxRowExt,
    WidgetExt,
    ContainerExt,
    StyleContextExt,
    StackExt,
    LabelExt,
    ImageExt,
    Inhibit,
};
use gdk::{
    NotifyType,
    EventType,
};
use news_flash::models::{
    ArticleID,
    Read,
    Marked,
};
use super::models::{
    ArticleListArticleModel,
};
use crate::util::{
    DateUtil,
    GtkUtil,
};
use failure::Error;
use failure::format_err;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use crate::Resources;
use crate::main_window::GtkHandle;

pub struct ArticleRow {
    article_id: ArticleID,
    widget: gtk::ListBoxRow,
}

impl ArticleRow {
    pub fn new(article: &ArticleListArticleModel) -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);

        let favicon : gtk::Image = builder.get_object("favicon").ok_or(format_err!("some err"))?;
        let article_eventbox : gtk::EventBox = builder.get_object("article_eventbox").ok_or(format_err!("some err"))?;
        let unread_eventbox : gtk::EventBox = builder.get_object("unread_eventbox").ok_or(format_err!("some err"))?;
        let marked_eventbox : gtk::EventBox = builder.get_object("marked_eventbox").ok_or(format_err!("some err"))?;
        let unread_stack : gtk::Stack = builder.get_object("unread_stack").ok_or(format_err!("some err"))?;
        let marked_stack : gtk::Stack = builder.get_object("marked_stack").ok_or(format_err!("some err"))?;
        let title_label : gtk::Label = builder.get_object("title_label").ok_or(format_err!("some err"))?;
        let summary_label : gtk::Label = builder.get_object("summary_label").ok_or(format_err!("some err"))?;
        let feed_label : gtk::Label = builder.get_object("feed_label").ok_or(format_err!("some err"))?;
        let date_label : gtk::Label = builder.get_object("date_label").ok_or(format_err!("some err"))?;
        let row = Self::create_row(&article_eventbox);

        let scale = match favicon.get_style_context() {
            Some(ctx) => ctx.get_scale(),
            None => 1,
        };

        let marked : gtk::Image = builder.get_object("marked").ok_or(format_err!("some err"))?;
        let marked_icon = Resources::get("icons/marked.svg").ok_or(format_err!("some err"))?;
        let surface = GtkUtil::create_surface_from_bytes(&marked_icon, 16, 16, scale)?;
        marked.set_from_surface(&surface);

        let unmarked : gtk::Image = builder.get_object("unmarked").ok_or(format_err!("some err"))?;
        let unmarked_icon = Resources::get("icons/unmarked.svg").ok_or(format_err!("some err"))?;
        let surface = GtkUtil::create_surface_from_bytes(&unmarked_icon, 16, 16, scale)?;
        unmarked.set_from_surface(&surface);

        let read : gtk::Image = builder.get_object("read").ok_or(format_err!("some err"))?;
        let read_icon = Resources::get("icons/read.svg").ok_or(format_err!("some err"))?;
        let surface = GtkUtil::create_surface_from_bytes(&read_icon, 16, 16, scale)?;
        read.set_from_surface(&surface);

        let unread : gtk::Image = builder.get_object("unread").ok_or(format_err!("some err"))?;
        let unread_icon = Resources::get("icons/unread.svg").ok_or(format_err!("some err"))?;
        let surface = GtkUtil::create_surface_from_bytes(&unread_icon, 16, 16, scale)?;
        unread.set_from_surface(&surface);

        title_label.set_text(&article.title);
        summary_label.set_text(&article.summary);
        feed_label.set_text(&article.feed_title);
        date_label.set_text(&DateUtil::format(&article.date));

        if let Some(icon) = &article.favicon {
            if let Some(data) = &icon.data {
                let surface = GtkUtil::create_surface_from_bytes(data, 16, 16, scale).unwrap();
                favicon.set_from_surface(&surface);
            }
        }

        let read_handle = Rc::new(RefCell::new(article.unread));
        let marked_handle = Rc::new(RefCell::new(article.marked));

        Self::setup_row_eventbox(&article_eventbox, &read_handle, &marked_handle, &unread_stack, &marked_stack, &title_label);
        Self::setup_unread_eventbox(&unread_eventbox, &read_handle, &unread_stack);
        Self::setup_marked_eventbox(&marked_eventbox, &marked_handle, &marked_stack);

        Ok(ArticleRow {
            article_id: article.id.clone(),
            widget: row,
        })
    }

    pub fn widget(&self) -> gtk::ListBoxRow {
        self.widget.clone()
    }

    fn create_row(widget: &gtk::EventBox) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(false);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context().unwrap();
        context.remove_class("activatable");

        row
    }

    fn setup_unread_eventbox(
        eventbox: &gtk::EventBox,
        read: &GtkHandle<Read>,
        unread_stack: &gtk::Stack,
    ) {
        let read_1 = read.clone();
        let stack_1 = unread_stack.clone();
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
        eventbox.connect_button_press_event(move |_widget, event| {
            if event.get_button() != 1 {
                return Inhibit(false)
            }
            match event.get_event_type() {
                EventType::ButtonRelease |
                EventType::DoubleButtonPress |
                EventType::TripleButtonPress => return Inhibit(false),
                _ => {},
            }
            let read = *read_3.borrow();
            match read {
                Read::Read => *read_3.borrow_mut() = Read::Unread,
                Read::Unread => *read_3.borrow_mut() = Read::Read,
            }
            Inhibit(true)
        });
    }

    fn setup_marked_eventbox(
        eventbox: &gtk::EventBox,
        marked: &GtkHandle<Marked>,
        marked_stack: &gtk::Stack,
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
        eventbox.connect_button_press_event(move |_widget, event| {
            if event.get_button() != 1 {
                return Inhibit(false)
            }
            match event.get_event_type() {
                EventType::ButtonRelease |
                EventType::DoubleButtonPress |
                EventType::TripleButtonPress => return Inhibit(false),
                _ => {},
            }
            let read = *marked_3.borrow();
            match read {
                Marked::Marked => *marked_3.borrow_mut() = Marked::Unmarked,
                Marked::Unmarked => *marked_3.borrow_mut() = Marked::Marked,
            }
            Inhibit(true)
        });
    }

    fn setup_row_eventbox(
        eventbox: &gtk::EventBox,
        read: &GtkHandle<Read>,
        marked: &GtkHandle<Marked>,
        unread_stack: &gtk::Stack,
        marked_stack: &gtk::Stack,
        title_label: &gtk::Label,
    ) {
        match *read.borrow() {
            Read::Read => unread_stack.set_visible_child_name("empty"),
            Read::Unread => {
                unread_stack.set_visible_child_name("unread");
                let context = title_label.get_style_context().unwrap();
                context.add_class("bold");
            },
        }

        match *marked.borrow() {
            Marked::Unmarked => marked_stack.set_visible_child_name("empty"),
            Marked::Marked => marked_stack.set_visible_child_name("marked"),
        }

        let read_1 = read.clone();
        let marked_1 = marked.clone();
        let unread_stack_1 = unread_stack.clone();
        let marked_stack_1 = marked_stack.clone();
        eventbox.connect_enter_notify_event(move |_widget, event| {
            if event.get_detail() == NotifyType::Inferior {
                return Inhibit(true)
            }
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
        eventbox.connect_leave_notify_event(move |_widget, event| {
            if event.get_detail() == NotifyType::Inferior {
                return Inhibit(true)
            }
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
}