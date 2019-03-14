mod change_set;

use std::collections::HashSet;
use failure::Error;
use failure::format_err;
pub use change_set::ArticleListChangeSet;
use news_flash::ArticleOrder;
use news_flash::models::{
    Article,
    ArticleID,
    FavIcon
};

#[derive(Debug)]
pub struct ArticleListModel {
    models: Vec<(Article, String, Option<FavIcon>)>,
    ids: HashSet<ArticleID>,
    sort: ArticleOrder,
}

impl ArticleListModel {
    pub fn new(sort: ArticleOrder) -> Self {
        ArticleListModel {
            models: Vec::new(),
            ids: HashSet::new(),
            sort: sort,
        }
    }

    pub fn add(&mut self, article: Article, feed_name: String, icon: Option<FavIcon>) -> Result<(), Error> {
        if self.ids.contains(&article.article_id) {
            return Err(format_err!("some err"))
        }
        self.ids.insert(article.article_id.clone());
        self.models.push((article, feed_name, icon));
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
            if let Some((new_article, new_feed_name, new_favicon)) = new_item {
                if old_item.is_none() {
                    new_index += 1;
                    diff.push(ArticleListChangeSet::Add(new_article.clone(), list_pos, new_feed_name.to_owned(), new_favicon.clone()));
                    list_pos += 1;
                    continue
                }
            }

            // remove all items after new_items ran out of items to compare
            if let Some((old_article, _, _)) = old_item {
                if new_item.is_none() {
                    diff.push(ArticleListChangeSet::Remove(old_article.article_id.clone()));
                    old_index += 1;
                    continue
                }
            }

            if let Some((old_article, _, _)) = old_item {
                if let Some((new_article, _, _)) = new_item {
                    // still the same item -> check for read & marked state
                    if new_article == old_article {
                        if new_article.unread != old_article.unread {
                            diff.push(ArticleListChangeSet::UpdateRead(new_article.article_id.clone(), new_article.unread));
                        }
                        if new_article.marked != old_article.marked {
                            diff.push(ArticleListChangeSet::UpdateMarked(new_article.article_id.clone(), new_article.marked));
                        }
                        list_pos += 1;
                        old_index += 1;
                        new_index += 1;
                        continue
                    }

                    // items differ -> remove old item and move on
                    diff.push(ArticleListChangeSet::Remove(old_article.article_id.clone()));
                    old_index += 1;
                    continue
                }
            }
        }

        diff
    }

    fn sort(&mut self) {
        match self.sort {
            ArticleOrder::OldestFirst => {
                self.models.sort_by(|(a, _, _), (b, _, _)| a.date.cmp(&b.date));
            },
            ArticleOrder::NewestFirst => {
                self.models.sort_by(|(a, _, _), (b, _, _)| a.date.cmp(&b.date).reverse());
            },
        }
    }

    pub fn calculate_selection(&mut self, selected_index: i32) -> Option<&Article> {
        self.sort();
        if let Some((index, (article, _, _))) = self.models.iter().enumerate().find(|(index, _)| index == &(selected_index as usize)) {
            return Some(article)
        }
        None
    }
}