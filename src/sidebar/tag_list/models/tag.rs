use news_flash::models::{Tag, TagID};
use std;
use std::cmp::Ordering;

#[derive(Eq, Clone, Debug)]
pub struct TagListTagModel {
    pub id: TagID,
    pub label: String,
    pub color: Option<String>,
    pub sort_index: i32,
}

impl TagListTagModel {
    pub fn new(tag: &Tag) -> Self {
        TagListTagModel {
            id: tag.tag_id.clone(),
            label: tag.label.clone(),
            color: tag.color.clone(),
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
