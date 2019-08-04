use news_flash::models::{CategoryID, FeedID};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum UndoAction {
    DeleteFeed(FeedID),
    DeleteCategory(CategoryID),
}
