mod change_set;

use std::collections::HashSet;
use failure::Error;
use failure::format_err;
use change_set::ArticleListChangeSet;
use news_flash::models::{
    Article,
    ArticleID,
};

#[derive(Debug)]
pub enum ArticleListSortBy {
    DateAsc,
    DateDesc,
}

#[derive(Debug)]
pub struct ArticleListModel {
    models: Vec<Article>,
    ids: HashSet<ArticleID>,
    sort: ArticleListSortBy,
}

impl ArticleListModel {
    pub fn new() -> Self {
        ArticleListModel {
            models: Vec::new(),
            ids: HashSet::new(),
            sort: ArticleListSortBy::DateDesc,
        }
    }

    pub fn add(&mut self, article: Article) -> Result<(), Error> {
        if self.ids.contains(&article.article_id) {
            return Err(format_err!("some err"))
        }
        self.ids.insert(article.article_id.clone());
        self.models.push(article);
        Ok(())
    }

    pub fn generate_diff(&mut self, other: &mut ArticleListModel) -> Vec<ArticleListChangeSet> {
        let mut diff : Vec<ArticleListChangeSet> = Vec::new();
        let mut list_pos = 0;
        let mut old_index = 0;
        let mut new_index = 0;
        self.sort();
        other.sort();
        let old_items = &mut self.models;
        let new_items = &mut other.models;
        
        loop {
            let old_item = old_items.get(old_index);
            let new_item = new_items.get(new_index);

            // iterated through both lists -> done
            if old_item.is_none() && new_item.is_none() {
                break
            }

            // add all items after old_items ran out of items to compare
            if let Some(new_item) = new_item {
                if old_item.is_none() {
                    new_index += 1;
                    diff.push(ArticleListChangeSet::Add(new_item.clone(), list_pos));
                    list_pos += 1;
                    continue
                }
            }

            // remove all items after new_items ran out of items to compare
            if let Some(old_item) = old_item {
                if new_item.is_none() {
                    diff.push(ArticleListChangeSet::Remove(old_item.article_id.clone()));
                    old_index += 1;
                    continue
                }
            }

            if let Some(old_item) = old_item {
                if let Some(new_item) = new_item {
                    // still the same item -> check for read & marked state
                    if new_item == old_item {
                        if new_item.unread != old_item.unread {
                            diff.push(ArticleListChangeSet::UpdateRead(new_item.article_id.clone(), new_item.unread));
                        }
                        if new_item.marked != old_item.marked {
                            diff.push(ArticleListChangeSet::UpdateMarked(new_item.article_id.clone(), new_item.marked));
                        }
                        list_pos += 1;
                        old_index += 1;
                        new_index += 1;
                        continue
                    }

                    // items differ -> remove old item and move on
                    diff.push(ArticleListChangeSet::Remove(old_item.article_id.clone()));
                    old_index += 1;
                    continue
                }
            }
        }

        diff
    }

    fn sort(&mut self) {
        match self.sort {
            ArticleListSortBy::DateAsc => {
                self.models.sort_by(|a, b| a.date.cmp(&b.date).reverse());
            },
            ArticleListSortBy::DateDesc => {
                self.models.sort_by(|a, b| a.date.cmp(&b.date));
            },
        }
    }

    pub fn calculate_selection(&mut self, selected_index: i32) -> Option<(usize, &Article)> {
        self.sort();
        self.models.iter().enumerate().find(|(index, _)| index == &(selected_index as usize))
    }
}