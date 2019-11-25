use news_flash::models::{CategoryID, FeedID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeedListDndAction {
    MoveFeed(FeedID, CategoryID, CategoryID, i32),
    MoveCategory(CategoryID, CategoryID, i32),
}
