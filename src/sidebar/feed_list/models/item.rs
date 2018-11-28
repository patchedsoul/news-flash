use std::cmp::Ordering;
use super::category::FeedListCategoryModel;
use super::feed::FeedListFeedModel;
use news_flash::models::{
    CategoryID,
    FeedID,
};

#[derive(Eq, Clone, Debug)]
pub enum FeedListItem {
    Feed(FeedListFeedModel),
    Category(FeedListCategoryModel),
}

impl PartialEq for FeedListItem {
    fn eq(&self, other: &FeedListItem) -> bool {
        match other {
            FeedListItem::Category(other_category) => {
                match self {
                    FeedListItem::Category(self_category) => {
                        other_category.id == self_category.id
                    },
                    FeedListItem::Feed(_) => false,
                }
            },
            FeedListItem::Feed(other_feed) => {
                match self {
                    FeedListItem::Feed(self_feed) => {
                        other_feed.id == self_feed.id
                    },
                    FeedListItem::Category(_) => false,
                }
            }
        }
    }
}

impl Ord for FeedListItem {
    fn cmp(&self, other: &FeedListItem) -> Ordering {
        match self {
            FeedListItem::Feed(self_feed) => {
                match other {
                    FeedListItem::Feed(other_feed) => self_feed.sort_index.cmp(&other_feed.sort_index),
                    FeedListItem::Category(other_category) => self_feed.sort_index.cmp(&other_category.sort_index),
                }
            },
            FeedListItem::Category(self_category) => {
                match other {
                    FeedListItem::Feed(other_feed) => self_category.sort_index.cmp(&other_feed.sort_index),
                    FeedListItem::Category(other_category) => self_category.sort_index.cmp(&other_category.sort_index),
                }
            },
        }
    }
}

impl PartialOrd for FeedListItem {
    fn partial_cmp(&self, other: &FeedListItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Eq, Clone, Debug)]
pub enum FeedListItemLight {
    Feed(FeedID),
    Category(CategoryID),
}

impl PartialEq for FeedListItemLight {
    fn eq(&self, other: &FeedListItemLight) -> bool {
        match other {
            FeedListItemLight::Category(other_category) => {
                match self {
                    FeedListItemLight::Category(self_category) => {
                        other_category == self_category
                    },
                    FeedListItemLight::Feed(_) => false,
                }
            },
            FeedListItemLight::Feed(other_feed) => {
                match self {
                    FeedListItemLight::Feed(self_feed) => {
                        other_feed == self_feed
                    },
                    FeedListItemLight::Category(_) => false,
                }
            }
        }
    }
}

