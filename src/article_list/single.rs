use gtk::{
    Builder,
    ContainerExt,
    ListBoxExt,
    WidgetExt,
};
use news_flash::models::{
    ArticleID,
    article::{
        Read,
        Marked,
    },
};
use super::models::ArticleListArticleModel;
use super::article_row::ArticleRow;
use std::collections::HashMap;
use std::str;
use failure::Error;
use failure::format_err;
use std::rc::Rc;
use std::cell::RefCell;
use crate::Resources;
use crate::util::GtkHandle;
use crate::gtk_handle;

pub struct SingleArticleList {
    scroll: gtk::ScrolledWindow,
    articles: HashMap<ArticleID, GtkHandle<ArticleRow>>,
    list: gtk::ListBox,
}

impl SingleArticleList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_list_single.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let scroll : gtk::ScrolledWindow = builder.get_object("article_list_scroll").ok_or(format_err!("some err"))?;
        let list : gtk::ListBox = builder.get_object("article_list_box").ok_or(format_err!("some err"))?;

        Ok(SingleArticleList {
            scroll: scroll,
            articles: HashMap::new(),
            list: list,
        })
    }

    pub fn widget(&self) -> gtk::ScrolledWindow {
        self.scroll.clone()
    }

    pub fn add(&mut self, article: &ArticleListArticleModel, pos: i32) {
        let article_row = ArticleRow::new(&article).unwrap();
        self.list.insert(&article_row.widget(), pos);
        article_row.widget().show();
        self.articles.insert(article.id.clone(), gtk_handle!(article_row));
    }

    pub fn remove(&mut self, id: ArticleID) {
        if let Some(article_row) = self.articles.get(&id) {
            self.list.remove(&article_row.borrow().widget());
        }
        let _ = self.articles.remove(&id);
    }

    pub fn update_marked(&mut self, id: ArticleID, marked: Marked) {
        if let Some(article_handle) = self.articles.get(&id) {
            article_handle.borrow_mut().update_marked(marked);
        }
    }

    pub fn update_read(&mut self, id: ArticleID, read: Read) {
        if let Some(article_handle) = self.articles.get(&id) {
            article_handle.borrow_mut().update_unread(read);
        }
    }
}