use news_flash::models::{CategoryID, FeedID};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeedListSelection {
    Cateogry(CategoryID),
    Feed(FeedID),
}
