use news_flash::models::{CategoryID, FeedID};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeedListDndAction {
    MoveFeed(FeedID, CategoryID, CategoryID, i32),
    MoveCategory(CategoryID, CategoryID, i32),
}
