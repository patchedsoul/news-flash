use crate::sidebar::FeedListItemID;
use news_flash::models::{CategoryID, FeedID, TagID};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SidebarSelection {
    All,
    Cateogry(CategoryID),
    Feed(FeedID),
    Tag(TagID),
}

impl SidebarSelection {
    pub fn from_feed_list_selection(selection: FeedListItemID) -> Self {
        match selection {
            FeedListItemID::Feed(id) => SidebarSelection::Feed(id),
            FeedListItemID::Category(id) => SidebarSelection::Cateogry(id),
        }
    }
}

impl PartialEq for SidebarSelection {
    fn eq(&self, other: &SidebarSelection) -> bool {
        match self {
            SidebarSelection::All => match other {
                SidebarSelection::All => true,
                _ => false,
            },
            SidebarSelection::Cateogry(self_id) => match other {
                SidebarSelection::Cateogry(other_id) => self_id == other_id,
                _ => false,
            },
            SidebarSelection::Feed(self_id) => match other {
                SidebarSelection::Feed(other_id) => self_id == other_id,
                _ => false,
            },
            SidebarSelection::Tag(self_id) => match other {
                SidebarSelection::Tag(other_id) => self_id == other_id,
                _ => false,
            },
        }
    }
}
