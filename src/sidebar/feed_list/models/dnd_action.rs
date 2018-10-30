use news_flash::models::{
    FeedID,
    CategoryID,
};

#[derive(Clone, Debug)]
pub enum FeedListRawDndAction {
    MoveFeedRaw(FeedID, i32),
    MoveCategoryRaw(CategoryID, i32),
}

#[derive(Clone, Debug)]
pub enum FeedListProcessedDndAction {
    MoveFeed(FeedID, CategoryID, i32),
    MoveCategory(CategoryID, CategoryID, i32),
}