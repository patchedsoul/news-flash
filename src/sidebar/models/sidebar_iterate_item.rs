use news_flash::models::{CategoryID, FeedID, TagID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SidebarIterateItem {
    SelectAll,
    FeedListSelectFirstItem,
    FeedListSelectLastItem,
    TagListSelectFirstItem,
    TagListSelectLastItem,
    SelectFeedListCategory(CategoryID),
    SelectFeedListFeed(FeedID),
    SelectTagList(TagID),
    NothingSelected,
}
