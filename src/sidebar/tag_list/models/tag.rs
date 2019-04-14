use news_flash::models::{Tag, TagID};
use std;
use std::cmp::Ordering;

pub static DEFAULT_COLOUR: &'static str = "#FF00FF";

#[derive(Eq, Clone, Debug)]
pub struct TagListTagModel {
    pub id: TagID,
    pub label: String,
    pub color: String,
    pub item_count: i32,
    pub sort_index: i32,
}

impl TagListTagModel {
    pub fn new(tag: &Tag, item_count: i32) -> Self {
        TagListTagModel {
            id: tag.tag_id.clone(),
            label: tag.label.clone(),
            color: match &tag.color {
                Some(colour) => colour.clone(),
                None => DEFAULT_COLOUR.to_owned(),
            },
            item_count: item_count,
            sort_index: match tag.sort_index {
                Some(index) => index,
                None => std::i32::MAX,
            },
        }
    }
}

impl PartialEq for TagListTagModel {
    fn eq(&self, other: &TagListTagModel) -> bool {
        self.id == other.id
    }
}

impl Ord for TagListTagModel {
    fn cmp(&self, other: &TagListTagModel) -> Ordering {
        self.sort_index.cmp(&other.sort_index)
    }
}

impl PartialOrd for TagListTagModel {
    fn partial_cmp(&self, other: &TagListTagModel) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
