use super::category::FeedListCategoryModel;
use super::change_set::FeedListChangeSet;
use super::error::{FeedListModelError, FeedListModelErrorKind};
use super::feed::FeedListFeedModel;
use super::item::FeedListItem;
use super::FeedListItemID;
use crate::sidebar::SidebarIterateItem;
use news_flash::models::{Category, CategoryID, FavIcon, Feed, FeedID, FeedMapping, NEWSFLASH_TOPLEVEL};

#[derive(Clone, Debug)]
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

    pub fn len(&self) -> i32 {
        let mut count = 0;
        Self::len_internal(&self.top_level, &mut count);
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

    pub fn add_feed(
        &mut self,
        feed: &Feed,
        mapping: &FeedMapping,
        item_count: i32,
        icon: Option<FavIcon>,
    ) -> Result<(), FeedListModelError> {
        if mapping.category_id == self.top_level_id {
            let contains_feed = self.top_level.iter().any(|item| {
                if let FeedListItem::Feed(item) = item {
                    return item.id == feed.feed_id;
                }
                false
            });
            if !contains_feed {
                let feed = FeedListFeedModel::new(feed, mapping, item_count, 0, icon);
                let item = FeedListItem::Feed(feed);
                self.top_level.push(item);
                self.top_level.sort();
            } else {
                return Err(FeedListModelErrorKind::AddDuplicateFeed)?;
            }
            return Ok(());
        }
        if let Some((parent, level)) = self.find_category(&mapping.category_id) {
            let feed = FeedListFeedModel::new(feed, mapping, item_count, level, icon);
            let item = FeedListItem::Feed(feed);
            parent.add_child(item);
            return Ok(());
        }

        Err(FeedListModelErrorKind::AddFeedNoParent)?
    }

    pub fn add_category(&mut self, category: &Category, item_count: i32) -> Result<(), FeedListModelError> {
        if category.parent_id == self.top_level_id {
            let contains_category = self.top_level.iter().any(|item| {
                if let FeedListItem::Category(item) = item {
                    return item.id == category.category_id;
                }
                false
            });
            if !contains_category {
                let category_ = FeedListCategoryModel::new(&category, item_count, 0);
                let item = FeedListItem::Category(category_);

                self.top_level.push(item);
                self.top_level.sort();
            } else {
                return Err(FeedListModelErrorKind::AddDuplicateCategory)?;
            }
            return Ok(());
        }
        if let Some((parent, level)) = self.find_category(&category.parent_id) {
            let category_ = FeedListCategoryModel::new(&category, item_count, level);
            let item = FeedListItem::Category(category_);
            parent.add_child(item);
            return Ok(());
        }

        Err(FeedListModelErrorKind::AddCategoryNoParent)?
    }

    fn find_category(&mut self, id: &CategoryID) -> Option<(&mut FeedListCategoryModel, i32)> {
        let mut level = 0;
        Self::search_subcategories(id, &mut self.top_level, &mut level)
    }

    fn search_subcategories<'a>(
        id: &CategoryID,
        children: &'a mut Vec<FeedListItem>,
        level: &mut i32,
    ) -> Option<(&'a mut FeedListCategoryModel, i32)> {
        *level += 1;
        for item in children {
            if let FeedListItem::Category(category) = item {
                let category_id = category.id.clone();
                if &category_id == id {
                    return Some((category, *level));
                } else if !category.children.is_empty() {
                    if let Some((category, level)) = Self::search_subcategories(id, &mut category.children, level) {
                        return Some((category, level));
                    }
                }
            }
        }
        *level -= 1;
        None
    }

    pub fn collapse_expand_category(
        &mut self,
        category_id: &CategoryID,
    ) -> Option<(Vec<FeedID>, Vec<CategoryID>, bool)> {
        if let Some((category, _)) = self.find_category(category_id) {
            let expanded = category.expand_collapse();
            let (feed_ids, category_ids) = Self::category_child_ids(&category);
            return Some((feed_ids, category_ids, expanded));
        }
        None
    }

    fn category_child_ids(category: &FeedListCategoryModel) -> (Vec<FeedID>, Vec<CategoryID>) {
        let mut feed_ids: Vec<FeedID> = Vec::new();
        let mut category_ids: Vec<CategoryID> = Vec::new();
        for item in &category.children {
            match item {
                FeedListItem::Feed(feed) => feed_ids.push(feed.id.clone()),
                FeedListItem::Category(category) => {
                    category_ids.push(category.id.clone());
                    if category.expanded {
                        let (mut sub_feeds, mut sub_categories) = Self::category_child_ids(&category);
                        feed_ids.append(&mut sub_feeds);
                        category_ids.append(&mut sub_categories);
                    }
                }
            }
        }
        (feed_ids, category_ids)
    }

    pub fn generate_diff(&self, other: &mut FeedListTree) -> Vec<FeedListChangeSet> {
        let mut list_pos = 0;
        Self::diff_level(&self.top_level, &mut other.top_level, &mut list_pos, true)
    }

    fn diff_level(
        old_items: &[FeedListItem],
        new_items: &mut [FeedListItem],
        list_pos: &mut i32,
        visible: bool,
    ) -> Vec<FeedListChangeSet> {
        let mut diff = Vec::new();
        let mut old_index = 0;
        let mut new_index = 0;
        loop {
            let old_item = old_items.get(old_index);
            let new_item = new_items.get_mut(new_index);

            // iterated through both lists -> done
            if old_item.is_none() && new_item.is_none() {
                break;
            }

            // remove all items after new_items ran out of items to compare
            if let Some(old_item) = old_item {
                if new_item.is_none() {
                    match old_item {
                        FeedListItem::Feed(old_feed) => diff.push(FeedListChangeSet::RemoveFeed(old_feed.id.clone())),
                        FeedListItem::Category(old_category) => {
                            diff.push(FeedListChangeSet::RemoveCategory(old_category.id.clone()));
                            if !old_category.children.is_empty() {
                                diff.append(&mut Self::diff_level(
                                    &old_category.children,
                                    &mut Vec::new(),
                                    list_pos,
                                    false,
                                ));
                            }
                        }
                    }
                    old_index += 1;
                    continue;
                }
            }

            if let Some(new_item) = new_item {
                // add all items after old_items ran out of items to compare
                if old_item.is_none() {
                    new_index += 1;
                    match new_item {
                        FeedListItem::Feed(new_feed) => {
                            diff.push(FeedListChangeSet::AddFeed(new_feed.clone(), *list_pos, visible));
                            *list_pos += 1;
                        }
                        FeedListItem::Category(ref mut new_category) => {
                            diff.push(FeedListChangeSet::AddCategory(new_category.clone(), *list_pos, visible));
                            *list_pos += 1;
                            if !new_category.children.is_empty() {
                                diff.append(&mut Self::diff_level(
                                    &Vec::new(),
                                    &mut new_category.children,
                                    list_pos,
                                    new_category.expanded,
                                ));
                            }
                        }
                    }

                    continue;
                }

                if let Some(old_item) = old_item {
                    // still the same item -> check for
                    if new_item == old_item {
                        match new_item {
                            FeedListItem::Feed(new_feed) => {
                                if let FeedListItem::Feed(old_feed) = old_item {
                                    if new_feed.item_count != old_feed.item_count {
                                        diff.push(FeedListChangeSet::FeedUpdateItemCount(
                                            new_feed.id.clone(),
                                            new_feed.item_count,
                                        ));
                                    }
                                    if new_feed.label != old_feed.label {
                                        diff.push(FeedListChangeSet::FeedUpdateLabel(
                                            new_feed.id.clone(),
                                            new_feed.label.clone(),
                                        ));
                                    }
                                }
                            }
                            FeedListItem::Category(new_category) => {
                                if let FeedListItem::Category(old_category) = old_item {
                                    new_category.expanded = old_category.expanded;
                                    if new_category.item_count != old_category.item_count {
                                        diff.push(FeedListChangeSet::CategoryUpdateItemCount(
                                            new_category.id.clone(),
                                            new_category.item_count,
                                        ));
                                    }
                                    if new_category.label != old_category.label {
                                        diff.push(FeedListChangeSet::CategoryUpdateLabel(
                                            new_category.id.clone(),
                                            new_category.label.clone(),
                                        ));
                                    }
                                }
                            }
                        }

                        // move 1 further on both lists and continue
                        *list_pos += 1;
                        old_index += 1;
                        new_index += 1;
                        match new_item {
                            FeedListItem::Category(new_category) => match old_item {
                                FeedListItem::Category(old_category) => {
                                    if !old_category.children.is_empty() || !new_category.children.is_empty() {
                                        diff.append(&mut Self::diff_level(
                                            &old_category.children,
                                            &mut new_category.children,
                                            list_pos,
                                            old_category.expanded,
                                        ));
                                    }
                                }
                                FeedListItem::Feed(_) => {
                                    if !new_category.children.is_empty() {
                                        diff.append(&mut Self::diff_level(
                                            &Vec::new(),
                                            &mut new_category.children,
                                            list_pos,
                                            new_category.expanded,
                                        ));
                                    }
                                }
                            },
                            FeedListItem::Feed(_) => match old_item {
                                FeedListItem::Category(old_category) => {
                                    if !old_category.children.is_empty() {
                                        diff.append(&mut Self::diff_level(
                                            &old_category.children,
                                            &mut Vec::new(),
                                            list_pos,
                                            false,
                                        ));
                                    }
                                }
                                FeedListItem::Feed(_) => {}
                            },
                        }
                        continue;
                    }

                    // items differ -> remove old item and move on
                    match old_item {
                        FeedListItem::Feed(old_feed) => diff.push(FeedListChangeSet::RemoveFeed(old_feed.id.clone())),
                        FeedListItem::Category(old_category) => {
                            diff.push(FeedListChangeSet::RemoveCategory(old_category.id.clone()));
                            if !old_category.children.is_empty() {
                                diff.append(&mut Self::diff_level(
                                    &old_category.children,
                                    &mut Vec::new(),
                                    list_pos,
                                    false,
                                ));
                            }
                        }
                    }
                    old_index += 1;
                    continue;
                }
            }
        }
        diff
    }

    pub fn calculate_dnd(&self, pos: i32) -> Result<(CategoryID, i32), FeedListModelError> {
        let mut pos_iter = 0;
        self.calc_dnd_subcategory(&self.top_level, &self.top_level_id, pos, &mut pos_iter)
    }

    fn calc_dnd_subcategory(
        &self,
        category: &[FeedListItem],
        parent_id: &CategoryID,
        list_pos: i32,
        global_pos_iter: &mut i32,
    ) -> Result<(CategoryID, i32), FeedListModelError> {
        if global_pos_iter == &list_pos {
            return Ok((parent_id.clone(), 0));
        }

        for (local_pos, item) in category.iter().enumerate() {
            *global_pos_iter += 1;

            if let FeedListItem::Category(model) = item {
                if let Ok((parent, pos)) =
                    self.calc_dnd_subcategory(&model.children, &model.id, list_pos, global_pos_iter)
                {
                    return Ok((parent, pos));
                }
            }

            if global_pos_iter == &list_pos {
                let parent = match item {
                    FeedListItem::Category(model) => model.parent_id.clone(),
                    FeedListItem::Feed(model) => model.parent_id.clone(),
                };
                return Ok((parent, local_pos as i32));
            }
        }
        Err(FeedListModelErrorKind::DnD)?
    }

    pub fn calculate_selection(&self, selected_index: i32) -> Option<(FeedListItemID, String)> {
        let mut index = 0;
        Self::calculate_selection_internal(selected_index + 1, &self.top_level, &mut index)
    }

    fn calculate_selection_internal(
        selected_index: i32,
        items: &[FeedListItem],
        index: &mut i32,
    ) -> Option<(FeedListItemID, String)> {
        for item in items {
            *index += 1;
            match item {
                FeedListItem::Feed(feed) => {
                    if *index == selected_index {
                        return Some((FeedListItemID::Feed(feed.id.clone()), feed.label.clone()));
                    }
                }
                FeedListItem::Category(category) => {
                    if *index == selected_index {
                        return Some((FeedListItemID::Category(category.id.clone()), category.label.clone()));
                    }
                    if let Some(selection) =
                        Self::calculate_selection_internal(selected_index, &category.children, index)
                    {
                        return Some(selection);
                    }
                }
            }
        }
        None
    }

    pub fn calculate_next_item(&mut self, selected_index: i32) -> SidebarIterateItem {
        let mut index = 0;
        let mut selected_found = false;
        self.top_level.sort();
        Self::iterate_next_internal(selected_index + 1, &mut self.top_level, &mut index, &mut selected_found)
    }

    fn iterate_next_internal(
        selected_index: i32,
        items: &[FeedListItem],
        index: &mut i32,
        selected_found: &mut bool,
    ) -> SidebarIterateItem {
        for item in items {
            *index += 1;
            match item {
                FeedListItem::Feed(feed) => {
                    if !*selected_found {
                        if *index == selected_index {
                            *selected_found = true;
                            continue;
                        }
                    } else {
                        return SidebarIterateItem::SelectFeedListFeed(feed.id.clone());
                    }
                }
                FeedListItem::Category(category) => {
                    if !*selected_found {
                        if *index == selected_index {
                            *selected_found = true;
                            if !category.expanded {
                                continue;
                            }
                        }
                    } else {
                        return SidebarIterateItem::SelectFeedListCategory(category.id.clone());
                    }

                    match Self::iterate_next_internal(selected_index, &category.children, index, selected_found) {
                        SidebarIterateItem::SelectFeedListCategory(category) => {
                            return SidebarIterateItem::SelectFeedListCategory(category)
                        }
                        SidebarIterateItem::SelectFeedListFeed(feed) => {
                            return SidebarIterateItem::SelectFeedListFeed(feed)
                        }
                        _ => {}
                    }
                }
            }
        }
        SidebarIterateItem::TagListSelectFirstItem
    }

    pub fn calculate_prev_item(&mut self, selected_index: i32) -> SidebarIterateItem {
        let mut index = self.len();
        let mut selected_found = false;
        self.top_level.sort();
        Self::iterate_prev_internal(selected_index, &mut self.top_level, &mut index, &mut selected_found)
    }

    fn iterate_prev_internal(
        selected_index: i32,
        items: &[FeedListItem],
        index: &mut i32,
        selected_found: &mut bool,
    ) -> SidebarIterateItem {
        for item in items.iter().rev() {
            match item {
                FeedListItem::Feed(feed) => {
                    *index -= 1;
                    if !*selected_found {
                        if *index == selected_index {
                            *selected_found = true;
                            continue;
                        }
                    } else {
                        return SidebarIterateItem::SelectFeedListFeed(feed.id.clone());
                    }
                }
                FeedListItem::Category(category) => {
                    if category.expanded {
                        match Self::iterate_prev_internal(selected_index, &category.children, index, selected_found) {
                            SidebarIterateItem::SelectFeedListCategory(category) => {
                                return SidebarIterateItem::SelectFeedListCategory(category)
                            }
                            SidebarIterateItem::SelectFeedListFeed(feed) => {
                                return SidebarIterateItem::SelectFeedListFeed(feed)
                            }
                            _ => {}
                        }
                    } else {
                        *index -= category.len();
                    }

                    *index -= 1;
                    if !*selected_found {
                        if *index == selected_index {
                            *selected_found = true;
                            if !category.expanded {
                                continue;
                            }
                        }
                    } else {
                        return SidebarIterateItem::SelectFeedListCategory(category.id.clone());
                    }
                }
            }
        }
        SidebarIterateItem::SelectAll
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        self.print_internal(&self.top_level, &mut 0);
    }

    fn print_internal(&self, category: &[FeedListItem], level: &mut i32) {
        let mut new_level = *level + 1;
        for item in category {
            for _ in 0..new_level {
                print!("-- ");
            }

            match item {
                FeedListItem::Category(model) => {
                    self.print_internal(&model.children, &mut new_level);
                }
                FeedListItem::Feed(model) => {
                    println!("{}", model.label);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sidebar::feed_list::models::{
        FeedListCategoryModel, FeedListChangeSet, FeedListFeedModel, FeedListTree,
    };
    use news_flash::models::{Category, CategoryID, Feed, FeedID, FeedMapping, NEWSFLASH_TOPLEVEL};

    fn building_blocks() -> (
        Category,
        Category,
        Category,
        Feed,
        Feed,
        FeedMapping,
        FeedMapping,
        FeedMapping,
    ) {
        let category_1 = Category {
            category_id: CategoryID::new("category_1"),
            label: "Cateogry 1".to_owned(),
            parent_id: NEWSFLASH_TOPLEVEL.clone(),
            sort_index: Some(0),
        };
        let category_2 = Category {
            category_id: CategoryID::new("category_2"),
            label: "Cateogry 2".to_owned(),
            parent_id: NEWSFLASH_TOPLEVEL.clone(),
            sort_index: Some(1),
        };
        let category_3 = Category {
            category_id: CategoryID::new("category_3"),
            label: "Cateogry 3".to_owned(),
            parent_id: NEWSFLASH_TOPLEVEL.clone(),
            sort_index: Some(2),
        };
        let feed_1 = Feed {
            feed_id: FeedID::new("feed_1"),
            label: "Feed 1".to_owned(),
            feed_url: None,
            icon_url: None,
            sort_index: Some(0),
            website: None,
        };
        let feed_2 = Feed {
            feed_id: FeedID::new("feed_2"),
            label: "Feed 2".to_owned(),
            feed_url: None,
            icon_url: None,
            sort_index: Some(1),
            website: None,
        };
        let mapping_1 = FeedMapping {
            feed_id: FeedID::new("feed_1"),
            category_id: CategoryID::new("category_2"),
        };
        let mapping_2 = FeedMapping {
            feed_id: FeedID::new("feed_1"),
            category_id: CategoryID::new("category_1"),
        };
        let mapping_3 = FeedMapping {
            feed_id: FeedID::new("feed_2"),
            category_id: CategoryID::new("category_3"),
        };

        (
            category_1, category_2, category_3, feed_1, feed_2, mapping_1, mapping_2, mapping_3,
        )
    }

    #[test]
    fn diff_tree_1() {
        let (mut category_1, mut category_2, mut category_3, feed_1, _feed_2, mapping_1, mapping_2, _mapping_3) =
            building_blocks();
        let mut old_tree = FeedListTree::new();
        old_tree.add_category(&category_1, 5).unwrap();
        old_tree.add_category(&category_2, 0).unwrap();
        old_tree.add_category(&category_3, 0).unwrap();
        old_tree.add_feed(&feed_1, &mapping_1, 1, None).unwrap();

        let mut new_tree = FeedListTree::new();
        category_1.label = "Category 1 new".to_owned();
        new_tree.add_category(&category_1, 2).unwrap();
        category_2.sort_index = Some(2);
        new_tree.add_category(&category_2, 0).unwrap();
        category_3.sort_index = Some(1);
        new_tree.add_category(&category_3, 1).unwrap();
        new_tree.add_feed(&feed_1, &mapping_2, 2, None).unwrap();

        let diff = old_tree.generate_diff(&mut new_tree);

        assert_eq!(diff.len(), 7);
        assert_eq!(
            diff.get(0),
            Some(&FeedListChangeSet::CategoryUpdateItemCount(
                category_1.category_id.clone(),
                2
            ))
        );
        assert_eq!(
            diff.get(1),
            Some(&FeedListChangeSet::CategoryUpdateLabel(
                category_1.category_id.clone(),
                "Category 1 new".to_owned()
            ))
        );
        assert_eq!(
            diff.get(2),
            Some(&FeedListChangeSet::AddFeed(
                FeedListFeedModel::new(&feed_1, &mapping_2, 2, 1, None),
                1,
                false
            ))
        );
        assert_eq!(
            diff.get(3),
            Some(&FeedListChangeSet::RemoveCategory(category_2.category_id.clone()))
        );
        assert_eq!(
            diff.get(4),
            Some(&FeedListChangeSet::RemoveFeed(feed_1.feed_id.clone()))
        );
        assert_eq!(
            diff.get(5),
            Some(&FeedListChangeSet::CategoryUpdateItemCount(
                category_3.category_id.clone(),
                1
            ))
        );
        assert_eq!(
            diff.get(6),
            Some(&FeedListChangeSet::AddCategory(
                FeedListCategoryModel::new(&category_2, 0, 0),
                3,
                false
            ))
        );
    }

    #[test]
    fn calc_dnd_1() {
        let (category_1, category_2, category_3, feed_1, feed_2, mapping_1, _, mapping_3) = building_blocks();

        let mut tree = FeedListTree::new();
        tree.add_category(&category_1, 5).unwrap();
        tree.add_category(&category_2, 0).unwrap();
        tree.add_category(&category_3, 0).unwrap();
        tree.add_feed(&feed_1, &mapping_1, 1, None).unwrap();
        tree.add_feed(&feed_2, &mapping_3, 1, None).unwrap();

        let (id, pos) = tree.calculate_dnd(2).unwrap();

        assert_eq!(id, CategoryID::new("category_2"));
        assert_eq!(pos, 0);
    }
}
