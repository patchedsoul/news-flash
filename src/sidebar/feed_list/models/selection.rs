use serde_derive::{Deserialize, Serialize};
use news_flash::models::{
    CategoryID,
    FeedID,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeedListSelection {
    Cateogry(CategoryID),
    Feed(FeedID),
}