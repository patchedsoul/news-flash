use super::item::FeedListItem;
use super::feed::FeedListFeedModel;
use super::category::FeedListCategoryModel;
use super::change_set::FeedListChangeSet;
use news_flash::models::{
    CategoryID,
    Category,
    NEWSFLASH_TOPLEVEL,
    FeedMapping,
    Feed,
};

pub struct FeedListTree {
    top_level_id: CategoryID,
    pub top_level: Vec<FeedListItem>,
}

impl FeedListTree {
    pub fn new() -> Self {
        FeedListTree {
            top_level_id: NEWSFLASH_TOPLEVEL.clone(),
            top_level: Vec::new(),
        }
    }

    pub fn add_feed(&mut self, feed: &Feed, mapping: &FeedMapping, item_count: i32) {
        if mapping.category_id == self.top_level_id {
            let feed = FeedListFeedModel::new(feed, mapping, item_count, 0);
            let item = FeedListItem::Feed(feed);
            self.top_level.push(item);
            self.top_level.sort();
            return
        }
        if let Some((parent, level)) = self.find_category(&mapping.category_id) {
            let feed = FeedListFeedModel::new(feed, mapping, item_count, level);
            let item = FeedListItem::Feed(feed);
            parent.add_child(item);
            return
        }

        // FIXME: else error!
    }

    pub fn add_category(&mut self, category: &Category, item_count: i32) {
        if category.parent == self.top_level_id {
            let category_ = FeedListCategoryModel::new(&category, item_count, 0);
            let item = FeedListItem::Category(category_);
            self.top_level.push(item);
            self.top_level.sort();
            return
        }
        if let Some((parent, level)) = self.find_category(&category.parent) {
            let category_ = FeedListCategoryModel::new(&category, item_count, level);
            let item = FeedListItem::Category(category_);
            parent.add_child(item);
            return
        }

        // FIXME: else error!
    }

    fn find_category(&mut self, id: &CategoryID) -> Option<(&mut FeedListCategoryModel, i32)> {
        let mut level = 0;
        Self::search_subcategories(id, &mut self.top_level, &mut level)
    }

    fn search_subcategories<'a>(id: &CategoryID, subcategories: &'a mut Vec<FeedListItem>, level: &mut i32) -> Option<(&'a mut FeedListCategoryModel, i32)> {
        *level += 1;
        for category in subcategories {
            if let FeedListItem::Category(category) = category {
                if &category.id == id {
                    return Some((category, *level))
                }
                else {
                    return Self::search_subcategories(id, &mut category.children, level)
                }
            }
        }
        None
    }

    pub fn generate_diff(&self, other: &FeedListTree) -> Vec<FeedListChangeSet> {
        let mut list_pos = 0;
        Self::diff_level(&self.top_level, &other.top_level, &mut list_pos)
    }

    fn diff_level(old_items: &Vec<FeedListItem>, new_items: &Vec<FeedListItem>, list_pos: &mut i32) -> Vec<FeedListChangeSet> {
        let mut diff = Vec::new();
        let mut old_index = 0;
        let mut new_index = 0;
        loop {
            let old_item = old_items.get(old_index);
            let new_item = new_items.get(new_index);

            // iterated through both lists -> done
            if old_item.is_none() && new_item.is_none() {
                break
            }
            
            if let Some(old_item) = old_item {
                if let Some(new_item) = new_item {
                    // still the same item -> check for
                    if new_item == old_item {
                        match new_item {
                            FeedListItem::Feed(new_feed) => {
                                match old_item {
                                    FeedListItem::Feed(old_feed) => {
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
                                    },
                                    _ => {},
                                }
                            },
                            FeedListItem::Category(new_category) => {
                                match old_item {
                                    FeedListItem::Category(old_category) => {
                                        if new_category.item_count != old_category.item_count {
                                            diff.push(FeedListChangeSet::CategoryUpdateItemCount(
                                                new_category.id.clone(),
                                                new_category.item_count));
                                        }
                                        if new_category.label != old_category.label {
                                            diff.push(FeedListChangeSet::CategoryUpdateLabel(
                                                new_category.id.clone(),
                                                new_category.label.clone()));
                                        }
                                    },
                                    _ => {},
                                }
                            },
                        }

                        // move 1 further on both lists and continue
                        old_index += 1;
                        new_index += 1;
                        *list_pos += 1;
                        continue
                    }

                    // item updated -> if item is category, recurse
                    match new_item {
                        FeedListItem::Category(new_category) => {
                            match old_item {
                                FeedListItem::Category(old_category) => {
                                    diff.append(&mut Self::diff_level(&old_category.children, &new_category.children, list_pos));
                                },
                                FeedListItem::Feed(_) => {
                                    diff.append(&mut Self::diff_level(&Vec::new(), &new_category.children, list_pos));
                                },
                            }
                        },
                        FeedListItem::Feed(_) => {
                            match old_item {
                                FeedListItem::Category(old_category) => {
                                    diff.append(&mut Self::diff_level(&old_category.children, &Vec::new(), list_pos));
                                },
                                FeedListItem::Feed(_) => {},
                            }
                        },
                    }
                }
                
                // items differ -> remove old item and move further down on old list
                match old_item {
                    FeedListItem::Feed(old_feed) => 
                        diff.push(FeedListChangeSet::RemoveFeed(old_feed.id.clone())),
                    FeedListItem::Category(old_category) => 
                        diff.push(FeedListChangeSet::RemoveCategory(old_category.id.clone()))
                }
                old_index += 1;
                continue
            }

            // add all items after old_items ran out of items to compare
            if let Some(new_item) = new_item {
                match new_item {
                    FeedListItem::Feed(new_feed) => 
                        diff.push(FeedListChangeSet::AddFeed(new_feed.clone(), *list_pos)),
                    FeedListItem::Category(new_category) => 
                        diff.push(FeedListChangeSet::AddCategory(new_category.clone(), *list_pos)),
                }
                new_index += 1;
                *list_pos += 1;
                continue
            }
        }
        diff
    }
}