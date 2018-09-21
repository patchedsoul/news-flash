use super::category::FeedListCategoryModel;
use super::feed::FeedListFeedModel;
use news_flash::models::{
    CategoryID,
    FeedID,
};

#[derive(Debug)]
pub enum FeedListChangeSet {
    RemoveFeed(FeedID),
    RemoveCategory(CategoryID),
    AddFeed(FeedListFeedModel, i32, bool),
    AddCategory(FeedListCategoryModel, i32, bool),
    FeedUpdateItemCount(FeedID, i32),
    CategoryUpdateItemCount(CategoryID, i32),
    FeedUpdateLabel(FeedID, String),
    CategoryUpdateLabel(CategoryID, String),
}

impl PartialEq for FeedListChangeSet {
    fn eq(&self, other: &FeedListChangeSet) -> bool {
        match self {
            FeedListChangeSet::RemoveFeed(id) => {
                match other {
                    FeedListChangeSet::RemoveFeed(other_id) => id == other_id,
                    _ => false,
                }
            },
            FeedListChangeSet::RemoveCategory(id) => {
                match other {
                    FeedListChangeSet::RemoveCategory(other_id) => id == other_id,
                    _ => false,
                }
            },
            FeedListChangeSet::AddFeed(model, pos, _) => {
                match other {
                    FeedListChangeSet::AddFeed(other_model, other_pos, _) => model.id == other_model.id && pos == other_pos,
                    _ => false,
                }
            },
            FeedListChangeSet::AddCategory(model, pos, _) => {
                match other {
                    FeedListChangeSet::AddCategory(other_model, other_pos, _) => model.id == other_model.id && pos == other_pos,
                    _ => false,
                }
            },
            FeedListChangeSet::FeedUpdateItemCount(id, count) => {
                match other {
                    FeedListChangeSet::FeedUpdateItemCount(other_id, other_count) => id == other_id && count == other_count,
                    _ => false,
                }
            },
            FeedListChangeSet::CategoryUpdateItemCount(id, count) => {
                match other {
                    FeedListChangeSet::CategoryUpdateItemCount(other_id, other_count) => id == other_id && count == other_count,
                    _ => false,
                }
            },
            FeedListChangeSet::FeedUpdateLabel(id, label) => {
                match other {
                    FeedListChangeSet::FeedUpdateLabel(other_id, other_label) => id == other_id && label == other_label,
                    _ => false,
                }
            },
            FeedListChangeSet::CategoryUpdateLabel(id, label) => {
                match other {
                    FeedListChangeSet::CategoryUpdateLabel(other_id, other_label) => id == other_id && label == other_label,
                    _ => false,
                }
            },
        }
    }
}