use super::tag::TagListTagModel;
use news_flash::models::{
    TagID,
};

#[derive(Debug)]
pub enum TagListChangeSet {
    Remove(TagID),
    Add(TagListTagModel, i32), // pos
    UpdateItemCount(TagID, i32), // Item count
    UpdateLabel(TagID, String), // label
}

impl PartialEq for TagListChangeSet {
    fn eq(&self, other: &TagListChangeSet) -> bool {
        match self {
            TagListChangeSet::Remove(id) => {
                match other {
                    TagListChangeSet::Remove(other_id) => id == other_id,
                    _ => false,
                }
            },
            TagListChangeSet::Add(model, pos) => {
                match other {
                    TagListChangeSet::Add(other_model, other_pos) => model.id == other_model.id && pos == other_pos,
                    _ => false,
                }
            },
            TagListChangeSet::UpdateItemCount(id, count) => {
                match other {
                    TagListChangeSet::UpdateItemCount(other_id, other_count) => id == other_id && count == other_count,
                    _ => false,
                }
            },
            TagListChangeSet::UpdateLabel(id, label) => {
                match other {
                    TagListChangeSet::UpdateLabel(other_id, other_label) => id == other_id && label == other_label,
                    _ => false,
                }
            },
        }
    }
}