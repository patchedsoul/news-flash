use serde_derive::{Deserialize, Serialize};
use news_flash::models::{
    CategoryID,
    FeedID,
    TagID,
};
use crate::sidebar::FeedListSelection;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SidebarSelection {
    All,
    Cateogry(CategoryID),
    Feed(FeedID),
    Tag(TagID),
}

impl SidebarSelection {
    pub fn from_feed_list_selection(selection: FeedListSelection) -> Self {
        match selection {
            FeedListSelection::Feed(id) => SidebarSelection::Feed(id),
            FeedListSelection::Cateogry(id) => SidebarSelection::Cateogry(id),
        }
    }
}