use super::category::FeedListCategoryModel;
use super::feed::FeedListFeedModel;
use news_flash::models::{
    CategoryID,
    FeedID,
};

pub enum FeedListChangeSet {
    RemoveFeed(FeedID),
    RemoveCategory(CategoryID),
    AddFeed(FeedListFeedModel, i32),
    AddCategory(FeedListCategoryModel, i32),
    FeedUpdateItemCount(FeedID, i32),
    CategoryUpdateItemCount(CategoryID, i32),
    FeedUpdateLabel(FeedID, String),
    CategoryUpdateLabel(CategoryID, String),
}