use super::item::FeedListItem;
use log::warn;
use news_flash::models::{Category, CategoryID};
use serde::{Deserialize, Serialize};
use std;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Eq, Clone, Debug, Serialize, Deserialize)]
pub struct FeedListCategoryModel {
    pub id: CategoryID,
    pub parent_id: CategoryID,
    pub label: String,
    pub sort_index: i32,
    pub children: Vec<FeedListItem>,
    pub level: i32,
    pub expanded: bool,
    pub item_count: i64,
}

impl FeedListCategoryModel {
    pub fn new(category: &Category, item_count: i64, level: i32) -> Self {
        FeedListCategoryModel {
            id: category.category_id.clone(),
            parent_id: category.parent_id.clone(),
            label: category.label.clone(),
            sort_index: match category.sort_index {
                Some(index) => index,
                None => std::i32::MAX,
            },
            children: Vec::new(),
            level,
            expanded: false,
            item_count,
        }
    }

    pub fn add_child(&mut self, item: FeedListItem) {
        let contains_item = self.children.iter().any(|i| match &item {
            FeedListItem::Feed(item) => match i {
                FeedListItem::Feed(i) => i.id == item.id,
                FeedListItem::Category(_) => false,
            },
            FeedListItem::Category(item) => match i {
                FeedListItem::Feed(_) => false,
                FeedListItem::Category(i) => i.id == item.id,
            },
        });
        if !contains_item {
            self.children.push(item);
            self.children.sort();
        } else {
            warn!("Category '{}' already contains item '{:?}'.", self.id, item);
        }
    }

    pub fn expand_collapse(&mut self) -> bool {
        self.expanded = !self.expanded;
        self.expanded
    }

    pub fn len(&self) -> i32 {
        let mut count = 0;
        Self::len_internal(&self.children, &mut count);
        count
    }

    fn len_internal(items: &[FeedListItem], count: &mut i32) {
        for item in items {
            *count += 1;
            if let FeedListItem::Category(category) = item {
                Self::len_internal(&category.children, count);
            }
        }
    }
}

impl Hash for FeedListCategoryModel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.parent_id.hash(state);
        self.item_count.hash(state);
        self.sort_index.hash(state);
    }
}

impl PartialEq for FeedListCategoryModel {
    fn eq(&self, other: &FeedListCategoryModel) -> bool {
        self.id == other.id //&& self.sort_index == other.sort_index
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

impl fmt::Display for FeedListCategoryModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}  (count: {}) (id: {}) (exp: {})",
            self.label, self.item_count, self.id, self.expanded
        )
    }
}
