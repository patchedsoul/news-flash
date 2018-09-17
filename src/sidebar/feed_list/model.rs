use std;
use news_flash::models::{
    CategoryID,
    Category,
    FeedID,
    Feed,
    FeedMapping,
    NEWSFLASH_TOPLEVEL,
    Url,
};

pub struct FeedListCategoryModel {
    pub id: CategoryID,
    pub parent_id: CategoryID,
    pub label: String,
    pub item_count: i32,
    pub sort_index: i32,
    pub feeds: Vec<FeedListFeedModel>,
    pub subcategories: Vec<FeedListCategoryModel>,
}

impl FeedListCategoryModel {
    pub fn new(category: &Category, item_count: i32) -> Self {
        FeedListCategoryModel {
            id: category.category_id.clone(),
            parent_id: category.parent.clone(),
            label: category.label.clone(),
            item_count: item_count,
            sort_index: match category.sort_index {
                Some(index) => index,
                None => std::i32::MAX,
            },
            feeds: Vec::new(),
            subcategories: Vec::new(),
        }
    }

    pub fn add_feed(&mut self, feed: FeedListFeedModel) {
        self.feeds.push(feed);
    }

    pub fn add_category(&mut self, category: FeedListCategoryModel) {
        self.subcategories.push(category);
    }
}

pub struct FeedListFeedModel {
    pub id: FeedID,
    pub parent_id: CategoryID,
    pub label: String,
    pub item_count: i32,
    pub sort_index: i32,
    pub icon: Option<Url>,
}

impl FeedListFeedModel {
    pub fn new(feed: &Feed, mapping: &FeedMapping, item_count: i32) -> Self {
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
        }
    }
}

pub struct FeedListTree {
    top_level_id: CategoryID,
    pub top_level_categories: Vec<FeedListCategoryModel>,
    pub top_level_feeds: Vec<FeedListFeedModel>,
}

impl FeedListTree {
    pub fn new() -> Self {
        FeedListTree {
            top_level_id: NEWSFLASH_TOPLEVEL.clone(),
            top_level_categories: Vec::new(),
            top_level_feeds: Vec::new(),
        }
    }

    pub fn add_category(&mut self, category: Category, item_count: i32) {
        if category.parent == self.top_level_id {
            self.top_level_categories.push(FeedListCategoryModel::new(&category, item_count));
        }
        if let Some(parent) = self.find_category(&category.parent) {
            parent.add_category(FeedListCategoryModel::new(&category, item_count));
        }

        // FIXME: else error!
    }

    fn find_category(&mut self, id: &CategoryID) -> Option<&mut FeedListCategoryModel> {
        Self::search_subcategories(id, &mut self.top_level_categories)
    }

    fn search_subcategories<'a>(id: &CategoryID, subcategories: &'a mut Vec<FeedListCategoryModel>) -> Option<&'a mut FeedListCategoryModel> {
        for category in subcategories {
            if &category.id == id {
                return Some(category)
            }
            else {
                return Self::search_subcategories(id, &mut category.subcategories)
            }
        }
        None
    }


}