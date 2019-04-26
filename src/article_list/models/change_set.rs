use super::article::ArticleListArticleModel;
use news_flash::models::{ArticleID, Marked, Read};
use std::fmt;

#[derive(Debug)]
pub enum ArticleListChangeSet<'a> {
    Remove(ArticleID),
    Add(&'a ArticleListArticleModel, i32), // pos
    UpdateRead(ArticleID, Read),
    UpdateMarked(ArticleID, Marked),
}

impl<'a> PartialEq for ArticleListChangeSet<'a> {
    fn eq(&self, other: &ArticleListChangeSet) -> bool {
        match self {
            ArticleListChangeSet::Remove(id) => match other {
                ArticleListChangeSet::Remove(other_id) => id == other_id,
                _ => false,
            },
            ArticleListChangeSet::Add(model, pos) => match other {
                ArticleListChangeSet::Add(other_model, other_pos) => model.id == other_model.id && pos == other_pos,
                _ => false,
            },
            ArticleListChangeSet::UpdateRead(id, read) => match other {
                ArticleListChangeSet::UpdateRead(other_id, other_read) => id == other_id && read == other_read,
                _ => false,
            },
            ArticleListChangeSet::UpdateMarked(id, marked) => match other {
                ArticleListChangeSet::UpdateMarked(other_id, other_marked) => id == other_id && marked == other_marked,
                _ => false,
            },
        }
    }
}

impl<'a> fmt::Display for ArticleListChangeSet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArticleListChangeSet::Add(model, pos) => write!(f, "Add id='{}' pos='{}'", model.id, pos),
            ArticleListChangeSet::Remove(id) => write!(f, "Remove id='{}'", id),
            ArticleListChangeSet::UpdateMarked(id, marked) => {
                write!(f, "UpdateMarked id='{}' marked='{:?}'", id, marked)
            }
            ArticleListChangeSet::UpdateRead(id, read) => write!(f, "UpdateMarked id='{}' read='{:?}'", id, read),
        }
    }
}
