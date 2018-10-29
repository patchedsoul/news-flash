use news_flash::models::{
    FeedID,
    CategoryID,
};

#[derive(Clone, Debug)]
pub enum FeedListDndAction {
    MoveFeedRaw(FeedID, i32),
    MoveCategoryRaw(CategoryID, i32),
    MoveFeed(FeedID, CategoryID, i32),
    MoveCategory(CategoryID, CategoryID, i32),
}