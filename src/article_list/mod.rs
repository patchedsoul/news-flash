mod single;
mod models;
mod article_row;

use gtk::{
    Builder,
    StackExt,
};
use single::SingleArticleList;
pub use models::ArticleListModel;
use models::ArticleListChangeSet;
use news_flash::ArticleOrder;
use std::str;
use failure::Error;
use failure::format_err;
use crate::Resources;
use std::rc::Rc;
use std::cell::RefCell;
use crate::main_window::GtkHandle;

pub struct ArticleList {
    stack: gtk::Stack,
    list_1: SingleArticleList,
    list_2: SingleArticleList,
    list_model: GtkHandle<ArticleListModel>,
}

impl ArticleList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_list.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let stack : gtk::Stack = builder.get_object("article_list_stack").ok_or(format_err!("some err"))?;

        let list_1 = SingleArticleList::new()?;
        let list_2 = SingleArticleList::new()?;

        let model = ArticleListModel::new(ArticleOrder::NewestFirst);

        stack.add_named(&list_1.widget(), "list_1");
        stack.add_named(&list_2.widget(), "list_2");

        Ok(ArticleList {
            stack: stack,
            list_1: list_1,
            list_2: list_2,
            list_model: Rc::new(RefCell::new(model)),
        })
    }

    pub fn widget(&self) -> gtk::Stack {
        self.stack.clone()
    }

    pub fn new_list(&self) {
        
    }

    pub fn update(&mut self, new_list: ArticleListModel) {
        let old_list = self.list_model.clone();
        let mut old_list = old_list.borrow_mut();
        self.list_model = Rc::new(RefCell::new(new_list));
        let mut new_list = self.list_model.borrow_mut();
        let list_diff = old_list.generate_diff(&mut new_list);

        for diff in list_diff {
            match diff {
                ArticleListChangeSet::Add(article, pos) => {
                    self.list_1.add(article, pos);
                },
                ArticleListChangeSet::Remove(id) => {
                    self.list_1.remove(id.clone());
                },
                ArticleListChangeSet::UpdateMarked(id, marked) => {
                    self.list_1.update_marked(id.clone(), marked);
                },
                ArticleListChangeSet::UpdateRead(id, read) => {
                    self.list_1.update_read(id.clone(), read);
                },
            }
        }
    }
}