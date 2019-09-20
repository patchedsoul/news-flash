mod change_set;
mod tag;

use crate::sidebar::SidebarIterateItem;
pub use change_set::TagListChangeSet;
use failure::format_err;
use failure::Error;
use news_flash::models::{Tag, TagID};
use std::collections::HashSet;
pub use tag::TagListTagModel;

#[derive(Clone, Debug)]
pub struct TagListModel {
    models: Vec<TagListTagModel>,
    tags: HashSet<TagID>,
}

impl TagListModel {
    pub fn new() -> Self {
        TagListModel {
            models: Vec::new(),
            tags: HashSet::new(),
        }
    }

    pub fn add(&mut self, tag: &Tag) -> Result<(), Error> {
        if self.tags.contains(&tag.tag_id) {
            return Err(format_err!("some err"));
        }
        let model = TagListTagModel::new(tag);
        self.tags.insert(model.id.clone());
        self.models.push(model);
        Ok(())
    }

    pub fn generate_diff(&mut self, other: &mut TagListModel) -> Vec<TagListChangeSet> {
        let mut diff: Vec<TagListChangeSet> = Vec::new();
        let mut list_pos = 0;
        let mut old_index = 0;
        let mut new_index = 0;
        self.sort();
        other.sort();
        let old_items = &mut self.models;
        let new_items = &mut other.models;
        loop {
            let old_item = old_items.get(old_index);
            let new_item = new_items.get(new_index);

            // iterated through both lists -> done
            if old_item.is_none() && new_item.is_none() {
                break;
            }

            // add all items after old_items ran out of items to compare
            if let Some(new_item) = new_item {
                if old_item.is_none() {
                    new_index += 1;
                    diff.push(TagListChangeSet::Add(new_item.clone(), list_pos));
                    list_pos += 1;
                    continue;
                }
            }

            // remove all items after new_items ran out of items to compare
            if let Some(old_item) = old_item {
                if new_item.is_none() {
                    diff.push(TagListChangeSet::Remove(old_item.id.clone()));
                    old_index += 1;
                    continue;
                }
            }

            if let Some(old_item) = old_item {
                if let Some(new_item) = new_item {
                    // still the same item -> check for item label
                    if new_item == old_item {
                        if new_item.label != old_item.label {
                            diff.push(TagListChangeSet::UpdateLabel(
                                new_item.id.clone(),
                                new_item.label.clone(),
                            ));
                        }
                        list_pos += 1;
                        old_index += 1;
                        new_index += 1;
                        continue;
                    }

                    // items differ -> remove old item and move on
                    diff.push(TagListChangeSet::Remove(old_item.id.clone()));
                    old_index += 1;
                    continue;
                }
            }
        }
        diff
    }

    fn sort(&mut self) {
        self.models.sort_by(|a, b| a.cmp(b));
    }

    pub fn calculate_selection(&self, selected_index: i32) -> Option<(usize, &TagListTagModel)> {
        self.models
            .iter()
            .enumerate()
            .find(|(index, _item)| index == &(selected_index as usize))
    }

    pub fn calculate_next_item(&self, selected_index: i32) -> SidebarIterateItem {
        match self.calculate_selection(selected_index + 1) {
            None => SidebarIterateItem::SelectAll,
            Some((_index, model)) => SidebarIterateItem::SelectTagList(model.id.clone()),
        }
    }

    pub fn calculate_prev_item(&self, selected_index: i32) -> SidebarIterateItem {
        match self.calculate_selection(selected_index - 1) {
            None => SidebarIterateItem::FeedListSelectLastItem,
            Some((_index, model)) => SidebarIterateItem::SelectTagList(model.id.clone()),
        }
    }

    pub fn first(&mut self) -> Option<TagListTagModel> {
        self.sort();
        self.models.first().cloned()
    }

    pub fn last(&mut self) -> Option<TagListTagModel> {
        self.sort();
        self.models.last().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::TagListChangeSet;
    use super::TagListModel;
    use news_flash::models::{Tag, TagID};

    #[test]
    fn taglist_diff_1() {
        let mut tag_1 = Tag {
            tag_id: TagID::new("Tag_1"),
            label: "label_1".to_owned(),
            color: None,
            sort_index: Some(0),
        };
        let mut tag_2 = Tag {
            tag_id: TagID::new("Tag_2"),
            label: "label_2".to_owned(),
            color: None,
            sort_index: Some(1),
        };
        let tag_3 = Tag {
            tag_id: TagID::new("Tag_3"),
            label: "label_3".to_owned(),
            color: None,
            sort_index: Some(2),
        };

        let mut old_list = TagListModel::new();
        old_list.add(&tag_1).expect("Failed to add tag_1 (old_list)");
        old_list.add(&tag_2).expect("Failed to add tag_2 (old_list)");
        old_list.add(&tag_3).expect("Failed to add tag_3 (old_list)");

        let mut new_list = TagListModel::new();
        tag_1.sort_index = Some(1);
        tag_2.sort_index = Some(0);
        new_list.add(&tag_1).expect("Failed to add tag_1 (new_list)");
        new_list.add(&tag_2).expect("Failed to add tag_2 (new_list)");
        new_list.add(&tag_3).expect("Failed to add tag_3 (new_list)");

        let diff = old_list.generate_diff(&mut new_list);
        assert_eq!(diff.len(), 5);
        assert_eq!(diff.get(0), Some(&TagListChangeSet::Remove(TagID::new("Tag_1"))));
    }
}
