use gtk::{
    Builder,
    ListBoxRowExt,
    WidgetExt,
    ContainerExt,
    StyleContextExt,
    LabelExt,
    ImageExt,
};
use news_flash::models::{
    ArticleID,
};
use super::models::ArticleListArticleModel;
use crate::util::{
    DateUtil,
    GtkUtil,
};
use failure::Error;
use failure::format_err;
use std::str;
use crate::Resources;

pub struct ArticleRow {
    article_id: ArticleID,
    widget: gtk::ListBoxRow,
    article_eventbox: gtk::EventBox,
    unread_eventbox: gtk::EventBox,
    marked_eventbox: gtk::EventBox,
    unread_stack: gtk::Stack,
    marked_stack: gtk::Stack,
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

        Ok(ArticleRow {
            article_id: article.id.clone(),
            widget: Self::create_row(&article_eventbox),
            article_eventbox: article_eventbox,
            unread_eventbox: unread_eventbox,
            marked_eventbox: marked_eventbox,
            unread_stack: unread_stack,
            marked_stack: marked_stack,
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
}