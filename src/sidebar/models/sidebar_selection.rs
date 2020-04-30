use crate::sidebar::FeedListItemID;
use news_flash::models::{CategoryID, FeedID, TagID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SidebarSelection {
    All,
    Category(CategoryID, String),
    Feed(FeedID, CategoryID, String),
    Tag(TagID, String),
}

impl SidebarSelection {
    pub fn from_feed_list_selection(selection: FeedListItemID, title: String) -> Self {
        match selection {
            FeedListItemID::Feed(id, parent) => SidebarSelection::Feed(id, parent, title),
            FeedListItemID::Category(id) => SidebarSelection::Category(id, title),
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
            SidebarSelection::Category(self_id, _title) => match other {
                SidebarSelection::Category(other_id, _title) => self_id == other_id,
                _ => false,
            },
            SidebarSelection::Feed(self_id, parent_id, _title) => match other {
                SidebarSelection::Feed(other_id, other_parent_id, _title) => {
                    self_id == other_id && parent_id == other_parent_id
                }
                _ => false,
            },
            SidebarSelection::Tag(self_id, _title) => match other {
                SidebarSelection::Tag(other_id, _title) => self_id == other_id,
                _ => false,
            },
        }
    }
}
