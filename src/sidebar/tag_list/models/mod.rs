mod tag;
mod change_set;

use failure::Error;
use failure::format_err;
use std::collections::HashMap;
use tag::TagListTagModel;
use change_set::TagListChangeSet;
use news_flash::models::{
    Tag,
    TagID,
};

struct TagList {
    tags: HashMap<TagID, TagListTagModel>,
}

impl TagList {
    pub fn new() -> Self {
        TagList {
            tags: HashMap::new(),
        }
    }

    pub fn add(&mut self, tag: &Tag, item_count: i32) -> Result<(), Error> {
        if self.tags.contains_key(&tag.tag_id) {
            return Err(format_err!("some err"))
        }
        let model = TagListTagModel::new(tag, item_count);
        self.tags.insert(tag.tag_id.clone(), model);
        Ok(())
    }

    fn get_map(&self) -> &HashMap<TagID, TagListTagModel> {
        &self.tags
    }

    pub fn generate_diff(&self, other: &TagList) -> Vec<TagListChangeSet> {
        let mut diff : Vec<TagListChangeSet> = Vec::new();
        let mut list_pos = 0;
        let mut old_index = 0;
        let mut new_index = 0;
        let map = other.get_map();
        let mut old_items: Vec<&TagListTagModel> = self.tags.iter().map(|(_, value)| value).collect();
        let mut new_items: Vec<&TagListTagModel> = map.iter().map(|(_, value)| value).collect();
        old_items.sort_by(|a, b| a.cmp(b));
        new_items.sort_by(|a, b| a.cmp(b));
        loop {
            let old_item = old_items.get(old_index);
            let new_item = new_items.get(new_index);

            // iterated through both lists -> done
            if old_item.is_none() && new_item.is_none() {
                break
            }

            // add all items after old_items ran out of items to compare
            if let Some(&new_item) = new_item {
                if old_item.is_none() {
                    new_index += 1;
                    diff.push(TagListChangeSet::Add(new_item.clone(), list_pos));
                    list_pos += 1;
                    continue
                }
            }

            // remove all items after new_items ran out of items to compare
            if let Some(old_item) = old_item {
                if new_item.is_none() {
                    diff.push(TagListChangeSet::Remove(old_item.id.clone()));
                    old_index += 1;
                    continue
                }
            }

            if let Some(old_item) = old_item {
                if let Some(new_item) = new_item {
                    // still the same item -> check for item count
                    if new_item == old_item {
                        if new_item.item_count != old_item.item_count {
                            diff.push(TagListChangeSet::UpdateItemCount(new_item.id.clone(), new_item.item_count));
                        }
                        if new_item.label != old_item.label {
                            diff.push(TagListChangeSet::UpdateLabel(new_item.id.clone(), new_item.label.clone()));
                        }
                        list_pos += 1;
                        old_index += 1;
                        new_index += 1;
                        continue
                    }

                    // items differ -> remove old item and move on
                    diff.push(TagListChangeSet::Remove(old_item.id.clone()));
                    old_index += 1;
                    continue
                }
            }
        }
        diff
    }
}

#[cfg(test)]
mod tests {
    use super::TagList;
    use news_flash::models::{
        Tag,
        TagID,
    };

    #[test]
    fn taglist_diff_1()
    {
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

        let mut old_list = TagList::new();
        old_list.add(&tag_1, 2).unwrap();
        old_list.add(&tag_2, 0).unwrap();
        old_list.add(&tag_3, 4).unwrap();

        let mut new_list = TagList::new();
        tag_1.sort_index = Some(1);
        tag_2.sort_index = Some(0);
        new_list.add(&tag_1, 1).unwrap();
        new_list.add(&tag_2, 1).unwrap();
        new_list.add(&tag_3, 0).unwrap();

        let diff = old_list.generate_diff(&new_list);
        println!("{:?}", diff);
    }
}