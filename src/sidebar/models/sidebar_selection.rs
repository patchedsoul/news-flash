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

impl PartialEq for SidebarSelection {
    fn eq(&self, other: &SidebarSelection) -> bool {
        match self {
            SidebarSelection::All => {
                match other {
                    SidebarSelection::All => true,
                    _ => false,
                }
            },
            SidebarSelection::Cateogry(self_id) => {
                match other {
                    SidebarSelection::Cateogry(other_id) => self_id == other_id,
                    _ => false,
                }
            },
            SidebarSelection::Feed(self_id) => {
                match other {
                    SidebarSelection::Feed(other_id) => self_id == other_id,
                    _ => false,
                }
            },
            SidebarSelection::Tag(self_id) => {
                match other {
                    SidebarSelection::Tag(other_id) => self_id == other_id,
                    _ => false,
                }
            }
        }
    }
}