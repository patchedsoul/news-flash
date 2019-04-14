use news_flash::models::{CategoryID, FeedID};

#[derive(Clone, Debug)]
pub enum FeedListDndAction {
    MoveFeed(FeedID, CategoryID, i32),
    MoveCategory(CategoryID, CategoryID, i32),
}
