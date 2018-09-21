use std;
use std::cmp::Ordering;
use super::item::FeedListItem;
use news_flash::models::{
    CategoryID,
    Category,
};

#[derive(Eq, Clone, Debug)]
pub struct FeedListCategoryModel {
    pub id: CategoryID,
    pub parent_id: CategoryID,
    pub label: String,
    pub item_count: i32,
    pub sort_index: i32,
    pub children: Vec<FeedListItem>,
    pub level: i32,
    pub expanded: bool,
}

impl FeedListCategoryModel {
    pub fn new(category: &Category, item_count: i32, level: i32) -> Self {
        FeedListCategoryModel {
            id: category.category_id.clone(),
            parent_id: category.parent.clone(),
            label: category.label.clone(),
            item_count: item_count,
            sort_index: match category.sort_index {
                Some(index) => index,
                None => std::i32::MAX,
            },
            children: Vec::new(),
            level: level,
            expanded: false,
        }
    }

    pub fn add_child(&mut self, item: FeedListItem) {
        let contains_item = self.children.iter().any(|i| {
            match &item {
                FeedListItem::Feed(item) => {
                    match i {
                        FeedListItem::Feed(i) => i.id == item.id,
                        FeedListItem::Category(_) => false,
                    }
                },
                FeedListItem::Category(item) => {
                    match i {
                        FeedListItem::Feed(_) => false,
                        FeedListItem::Category(i) => i.id == item.id,
                    }
                },
            }
        });
        if !contains_item {
            self.children.push(item);
            self.children.sort();
        }
        else {
            // FIXME: warn/error
        }
    }

    pub fn expand_collapse(&mut self) -> bool {
        self.expanded = !self.expanded;
        self.expanded
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