mod single;
mod models;

use gtk::{
    Builder,
    StackExt,
};
use single::SingleArticleList;
use models::ArticleListModel;
use std::str;
use failure::Error;
use failure::format_err;
use crate::Resources;

pub struct ArticleList {
    stack: gtk::Stack,
    list_1: SingleArticleList,
    list_2: SingleArticleList,
    model: ArticleListModel,
}

impl ArticleList {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/article_list.ui").ok_or(format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let stack : gtk::Stack = builder.get_object("article_list_stack").ok_or(format_err!("some err"))?;

        let list_1 = SingleArticleList::new()?;
        let list_2 = SingleArticleList::new()?;

        let model = ArticleListModel::new();

        stack.add_named(&list_1.widget(), "list_1");
        stack.add_named(&list_2.widget(), "list_2");

        Ok(ArticleList {
            stack: stack,
            list_1: list_1,
            list_2: list_2,
            model: model,
        })
    }

    pub fn widget(&self) -> gtk::Stack {
        self.stack.clone()
    }
}