use std;
use std::cmp::Ordering;
use news_flash::models::{
    CategoryID,
    FeedID,
    Feed,
    FeedMapping,
    Url,
};

#[derive(Eq, Clone, Debug)]
pub struct FeedListFeedModel {
    pub id: FeedID,
    pub parent_id: CategoryID,
    pub label: String,
    pub item_count: i32,
    pub sort_index: i32,
    pub icon: Option<Url>,
    pub level: i32,
}

impl FeedListFeedModel {
    pub fn new(feed: &Feed, mapping: &FeedMapping, item_count: i32, level: i32) -> Self {
        FeedListFeedModel {
            id: feed.feed_id.clone(),
            parent_id: mapping.category_id.clone(),
            label: feed.label.clone(),
            item_count: item_count,
            sort_index: match feed.sort_index {
                Some(index) => index,
                None => std::i32::MAX,
            },
            icon: feed.icon_url.clone(),
            level: level,
        }
    }
}

impl PartialEq for FeedListFeedModel {
    fn eq(&self, other: &FeedListFeedModel) -> bool {
        self.id == other.id && self.sort_index == other.sort_index
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