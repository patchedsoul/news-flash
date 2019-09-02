use crate::sidebar::FeedListCountType;
use news_flash::models::{CategoryID, FavIcon, Feed, FeedID, FeedMapping};
use std;
use std::cmp::Ordering;
use std::fmt;

#[derive(Eq, Clone, Debug)]
pub struct FeedListFeedModel {
    pub id: FeedID,
    pub parent_id: CategoryID,
    pub label: String,
    pub sort_index: i32,
    pub icon: Option<FavIcon>,
    pub level: i32,
    unread_count: i32,
    marked_count: i32,
}

impl FeedListFeedModel {
    pub fn new(
        feed: &Feed,
        mapping: &FeedMapping,
        unread_count: i32,
        marked_count: i32,
        level: i32,
        icon: Option<FavIcon>,
    ) -> Self {
        FeedListFeedModel {
            id: feed.feed_id.clone(),
            parent_id: mapping.category_id.clone(),
            label: feed.label.clone(),
            sort_index: match feed.sort_index {
                Some(index) => index,
                None => std::i32::MAX,
            },
            icon,
            level,
            unread_count,
            marked_count,
        }
    }

    pub fn get_item_count_for_type(&self, count_type: &FeedListCountType) -> i32 {
        match count_type {
            FeedListCountType::Unread => self.unread_count,
            FeedListCountType::Marked => self.marked_count,
        }
    }

    pub fn set_item_count(&mut self, count: i32, count_type: &FeedListCountType) {
        match count_type {
            FeedListCountType::Unread => self.unread_count = count,
            FeedListCountType::Marked => self.marked_count = count,
        }
    }
}

impl PartialEq for FeedListFeedModel {
    fn eq(&self, other: &FeedListFeedModel) -> bool {
        self.id == other.id //&& self.sort_index == other.sort_index
    }
}

impl Ord for FeedListFeedModel {
    fn cmp(&self, other: &FeedListFeedModel) -> Ordering {
        self.sort_index.cmp(&other.sort_index)
    }
}

impl PartialOrd for FeedListFeedModel {
    fn partial_cmp(&self, other: &FeedListFeedModel) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for FeedListFeedModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (unread: {}) (marked: {}) (id: {})",
            self.label, self.unread_count, self.marked_count, self.id
        )
    }
}
