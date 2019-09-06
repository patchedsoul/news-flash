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
    pub item_count: i64,
}

impl FeedListFeedModel {
    pub fn new(
        feed: &Feed,
        mapping: &FeedMapping,
        item_count: i64,
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
            item_count,
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
            "{} (count: {}) (id: {})",
            self.label, self.item_count, self.id
        )
    }
}
