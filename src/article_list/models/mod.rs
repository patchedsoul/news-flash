mod article;
mod article_update_msg;
mod change_set;
mod error;

use crate::content_page::HeaderSelection;
pub use article::ArticleListArticleModel;
pub use article_update_msg::{MarkUpdate, ReadUpdate};
pub use change_set::ArticleListChangeSet;
use error::{ArticleListModelError, ArticleListModelErrorKind};
use log::warn;
use news_flash::models::{Article, ArticleID, ArticleOrder, Feed, Marked, Read};
use std::collections::HashSet;

#[derive(Debug)]
pub struct ArticleListModel {
    models: Vec<ArticleListArticleModel>,
    ids: HashSet<ArticleID>,
    sort: ArticleOrder,
}

impl ArticleListModel {
    pub fn new(sort: &ArticleOrder) -> Self {
        ArticleListModel {
            models: Vec::new(),
            ids: HashSet::new(),
            sort: sort.clone(),
        }
    }

    pub fn order(&self) -> ArticleOrder {
        self.sort.clone()
    }

    pub fn add(&mut self, article: Article, feed: &Feed) -> Result<(), ArticleListModelError> {
        if self.contains(&article.article_id) {
            warn!("Listmodel already contains id {}", article.article_id);
            return Err(ArticleListModelErrorKind::AlreadyContainsArticle.into());
        }
        self.ids.insert(article.article_id.clone());
        self.models.push(ArticleListArticleModel::new(article, feed));
        Ok(())
    }

    pub fn add_model(&mut self, model: ArticleListArticleModel) -> Result<(), ArticleListModelError> {
        if self.contains(&model.id) {
            warn!("Listmodel already contains id {}", model.id);
            return Err(ArticleListModelErrorKind::AlreadyContainsArticle.into());
        }
        self.ids.insert(model.id.clone());
        self.models.push(model);
        Ok(())
    }

    pub fn contains(&self, article_id: &ArticleID) -> bool {
        self.ids.contains(article_id)
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn models(&self) -> &Vec<ArticleListArticleModel> {
        &self.models
    }

    pub fn set_read(&mut self, id: &ArticleID, read: Read) {
        if !self.contains(id) {
            return;
        }

        if let Some(article_model) = self.models.iter_mut().find(|a| &a.id == id) {
            article_model.read = read;
        }
    }

    pub fn set_marked(&mut self, id: &ArticleID, marked: Marked) {
        if !self.contains(id) {
            return;
        }

        if let Some(article_model) = self.models.iter_mut().find(|a| &a.id == id) {
            article_model.marked = marked;
        }
    }

    pub fn get_relevant_count(&self, header_selection: &HeaderSelection) -> usize {
        match header_selection {
            HeaderSelection::All => self.models.len(),
            HeaderSelection::Unread => self.models.iter().filter(|a| a.read == Read::Unread).count(),
            HeaderSelection::Marked => self.models.iter().filter(|a| a.marked == Marked::Marked).count(),
        }
    }

    pub fn generate_diff<'a>(&'a mut self, new_list: &'a mut ArticleListModel) -> Vec<ArticleListChangeSet> {
        let mut diff: Vec<ArticleListChangeSet> = Vec::new();
        let mut list_pos = 0;
        let mut old_index = 0;
        let mut new_index = 0;
        self.sort();
        new_list.sort();
        let old_items = &mut self.models;
        let new_items = &mut new_list.models;
        let new_articles = &mut new_list.ids;

        loop {
            let old_item = old_items.get(old_index);
            let new_item = new_items.get(new_index);

            // iterated through both lists -> done
            if old_item.is_none() && new_item.is_none() {
                break;
            }

            // add all items after old_items ran out of items to compare
            if let Some(new_model) = new_item {
                if old_item.is_none() {
                    new_index += 1;
                    diff.push(ArticleListChangeSet::Add(&new_model, list_pos));
                    list_pos += 1;
                    continue;
                }
            }

            // remove all items after new_items ran out of items to compare
            if let Some(old_model) = old_item {
                if new_item.is_none() {
                    diff.push(ArticleListChangeSet::Remove(old_model.id.clone()));
                    old_index += 1;
                    continue;
                }
            }

            if let Some(old_model) = old_item {
                if let Some(new_model) = new_item {
                    // still the same item -> check for read & marked state
                    if new_model == old_model {
                        if new_model.read != old_model.read {
                            diff.push(ArticleListChangeSet::UpdateRead(new_model.id.clone(), new_model.read));
                        }
                        if new_model.marked != old_model.marked {
                            diff.push(ArticleListChangeSet::UpdateMarked(
                                new_model.id.clone(),
                                new_model.marked,
                            ));
                        }
                        list_pos += 1;
                        old_index += 1;
                        new_index += 1;
                        continue;
                    }

                    if new_articles.contains(&old_model.id) {
                        diff.push(ArticleListChangeSet::Add(&new_model, list_pos));
                        list_pos += 1;
                        new_index += 1;
                        continue;
                    }

                    // items differ -> remove old item and move on
                    diff.push(ArticleListChangeSet::Remove(old_model.id.clone()));
                    old_index += 1;
                    continue;
                }
            }
        }

        diff
    }

    fn sort(&mut self) {
        match self.sort {
            ArticleOrder::OldestFirst => {
                self.models.sort_by(|a, b| a.date.cmp(&b.date));
            }
            ArticleOrder::NewestFirst => {
                self.models.sort_by(|a, b| a.date.cmp(&b.date).reverse());
            }
        }
    }

    pub fn first(&mut self) -> Option<&ArticleListArticleModel> {
        self.sort();
        self.models.iter().next()
    }

    pub fn calculate_selection(&mut self, selected_index: i32) -> Option<&ArticleListArticleModel> {
        self.sort();
        if let Some((_index, article)) = self
            .models
            .iter()
            .enumerate()
            .find(|(index, _)| index == &(selected_index as usize))
        {
            return Some(article);
        }
        None
    }
}
