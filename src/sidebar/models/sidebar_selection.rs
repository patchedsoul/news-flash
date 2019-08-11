use crate::sidebar::FeedListItemID;
use news_flash::models::{CategoryID, FeedID, TagID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SidebarSelection {
    All,
    Cateogry((CategoryID, String)),
    Feed((FeedID, String)),
    Tag((TagID, String)),
}

impl SidebarSelection {
    pub fn from_feed_list_selection(selection: FeedListItemID, title: String) -> Self {
        match selection {
            FeedListItemID::Feed(id) => SidebarSelection::Feed((id, title)),
            FeedListItemID::Category(id) => SidebarSelection::Cateogry((id, title)),
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
            SidebarSelection::Cateogry((self_id, _title)) => match other {
                SidebarSelection::Cateogry((other_id, _title)) => self_id == other_id,
                _ => false,
            },
            SidebarSelection::Feed((self_id, _title)) => match other {
                SidebarSelection::Feed((other_id, _title)) => self_id == other_id,
                _ => false,
            },
            SidebarSelection::Tag((self_id, _title)) => match other {
                SidebarSelection::Tag((other_id, _title)) => self_id == other_id,
                _ => false,
            },
        }
    }
}
