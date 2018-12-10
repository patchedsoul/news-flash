use news_flash::models::{
    FeedID,
    CategoryID,
};

#[derive(Clone, Debug)]
pub enum FeedListDndAction {
    MoveFeed(FeedID, CategoryID, i32),
    MoveCategory(CategoryID, CategoryID, i32),
}