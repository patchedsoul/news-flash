use gtk::{
    Builder,
    ListBoxRowExt,
    WidgetExt,
    ContainerExt,
    StyleContextExt,
    LabelExt,
};
use news_flash::models::{
    Article,
    ArticleID,
    FavIcon,
};
use crate::util::DateUtil;
use failure::Error;
use failure::format_err;
use std::str;
use crate::Resources;

pub struct ArticleRow {
    article_id: ArticleID,
    widget: gtk::ListBoxRow,
    favicon: gtk::Image,
    article_eventbox: gtk::EventBox,
    unread_eventbox: gtk::EventBox,
    marked_eventbox: gtk::EventBox,
    unread_stack: gtk::Stack,
    marked_stack: gtk::Stack,
    title_label: gtk::Label,
    summary_label: gtk::Label,
    feed_label: gtk::Label,
    date_label: gtk::Label,
}

impl ArticleRow {
    pub fn new(article: &Article, feed_name: String, icon: Option<FavIcon>) -> Result<Self, Error> {
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

        if let Some(title) = &article.title {
            title_label.set_text(&title);
        }
        
        if let Some(summary) = &article.summary {
            summary_label.set_text(summary);
        }

        feed_label.set_text(&feed_name);
        date_label.set_text(&DateUtil::format(&article.date));

        Ok(ArticleRow {
            article_id: article.article_id.clone(),
            widget: Self::create_row(&article_eventbox),
            favicon: favicon,
            article_eventbox: article_eventbox,
            unread_eventbox: unread_eventbox,
            marked_eventbox: marked_eventbox,
            unread_stack: unread_stack,
            marked_stack: marked_stack,
            title_label: title_label,
            summary_label: summary_label,
            feed_label: feed_label,
            date_label: date_label,
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