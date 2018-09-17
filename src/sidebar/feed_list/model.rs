use std;
use std::cmp::Ordering;
use news_flash::models::{
    CategoryID,
    Category,
    FeedID,
    Feed,
    FeedMapping,
    NEWSFLASH_TOPLEVEL,
    Url,
};

pub enum FeedListChangeSet {
    RemoveFeed(FeedID),
    RemoveCategory(CategoryID),
    AddFeed(FeedListFeedModel, i32),
    AddCategory(FeedListCategoryModel, i32),
    FeedUpdateItemCount(FeedID, i32),
    CategoryUpdateItemCount(CategoryID, i32),
    FeedUpdateLabel(FeedID, String),
    CategoryUpdateLabel(CategoryID, String),
}

#[derive(Eq)]
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
        self.feeds.sort();
    }

    pub fn add_category(&mut self, category: FeedListCategoryModel) {
        self.subcategories.push(category);
        self.subcategories.sort();
    }
}

impl PartialEq for FeedListCategoryModel {
    fn eq(&self, other: &FeedListCategoryModel) -> bool {
        self.id == other.id && self.sort_index == other.sort_index
    }
}

impl Ord for FeedListCategoryModel {
    fn cmp(&self, other: &FeedListCategoryModel) -> Ordering {
        self.sort_index.cmp(&other.sort_index)
    }
}

impl PartialOrd for FeedListCategoryModel {
    fn partial_cmp(&self, other: &FeedListCategoryModel) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, Clone)]
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

    pub fn add_feed(&mut self, feed: &Feed, mapping: &FeedMapping, item_count: i32) {
        if mapping.category_id == self.top_level_id {
            self.top_level_feeds.push(FeedListFeedModel::new(feed, mapping, item_count));
            self.top_level_feeds.sort();
            return
        }
        if let Some(parent) = self.find_category(&mapping.category_id) {
            parent.add_feed(FeedListFeedModel::new(feed, mapping, item_count));
            return
        }

        // FIXME: else error!
    }

    pub fn add_category(&mut self, category: &Category, item_count: i32) {
        if category.parent == self.top_level_id {
            self.top_level_categories.push(FeedListCategoryModel::new(&category, item_count));
            self.top_level_categories.sort();
            return
        }
        if let Some(parent) = self.find_category(&category.parent) {
            parent.add_category(FeedListCategoryModel::new(&category, item_count));
            return
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

    pub fn generate_diff(&self, other: &FeedListTree) -> Vec<FeedListChangeSet> {
        let diff = Vec::new();
        let mut list_pos = 0;
        

        diff
    }

    fn diff_feeds(old_feeds: &Vec<FeedListFeedModel>, new_feeds: &Vec<FeedListFeedModel>, list_pos: &mut i32) -> Vec<FeedListChangeSet> {
        let mut diff = Vec::new();
        let mut old_index = 0;
        let mut new_index = 0;
        loop {
            let old_feed = old_feeds.get(old_index);
            let new_feed = new_feeds.get(new_index);

            // iterated through both lists -> done
            if old_feed.is_none() && new_feed.is_none() {
                break
            }
            
            if let Some(old_feed) = old_feed {
                if let Some(new_feed) = new_feed {
                    // still the same feed -> check for
                    if new_feed.id == old_feed.id {
                        if new_feed.item_count != old_feed.item_count {
                            diff.push(FeedListChangeSet::FeedUpdateItemCount(
                                new_feed.id.clone(),
                                new_feed.item_count));
                        }
                        if new_feed.label != old_feed.label {
                            diff.push(FeedListChangeSet::FeedUpdateLabel(
                                new_feed.id.clone(),
                                new_feed.label.clone()));
                        }

                        // move 1 further on both lists and continue
                        old_index += 1;
                        new_index += 1;
                        *list_pos += 1;
                        continue
                    }
                }
                
                // feeds differ -> remove old feed and move further down on old list
                diff.push(FeedListChangeSet::RemoveFeed(old_feed.id.clone()));
                old_index += 1;
                continue
            }

            // add all feeds after old_feeds ran out of items to compare
            if let Some(new_feed) = new_feed {
                diff.push(FeedListChangeSet::AddFeed(new_feed.clone(), *list_pos));
                new_index += 1;
                *list_pos += 1;
                continue
            }
        }
        diff
    }
}