use Resources;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use news_flash::models::{
    Category as CategoryModel,
    Feed as FeedModel,
    CategoryID,
    FeedID,
    NEWSFLASH_TOPLEVEL,
};
use sidebar::{
    CategoryRow,
    FeedRow,
};
use gtk::{
    self,
    ListBoxExt,
    WidgetExt,
};
use gdk::{
    EventType,
};

type Handle<T> = Rc<RefCell<T>>;
type HandleMap<T, K> = Handle<HashMap<T, K>>;


#[derive(Clone, Debug)]
pub struct FeedList {
    pub(crate) widget: gtk::ListBox,
    categories: HandleMap<CategoryID, Handle<CategoryRow>>,
    feeds: HandleMap<FeedID, Handle<FeedRow>>,
    mappings: HandleMap<CategoryID, Vec<FeedID>>,
}

impl FeedList {
    pub fn new() -> Self {
        let ui_data = Resources::get("ui/feedlist.ui").unwrap();
        let ui_string = str::from_utf8(&ui_data).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let feed_list : gtk::ListBox = builder.get_object("feed_list").unwrap();

        FeedList {
            widget: feed_list,
            categories: Rc::new(RefCell::new(HashMap::new())),
            feeds: Rc::new(RefCell::new(HashMap::new())),
            mappings: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn add_category(&mut self, category: &CategoryModel) {
        let level = self.calculate_level(&category.parent);
        let category_widget = CategoryRow::new(category, level);
        let mappings = self.mappings.clone();
        let feeds = self.feeds.clone();
        let categories = self.categories.clone();
        let category_id = category.category_id.clone();
        let pos = self.calculate_position(category.sort_index, &category.parent);
        self.widget.insert(&category_widget.borrow().row(), pos);
        {
            categories.borrow_mut().insert(category.category_id.clone(), category_widget.clone());
        }
        category_widget.borrow().expander_event().connect_button_press_event(move |_widget, event| {
            if event.get_event_type() == EventType::ButtonPress {
                Self::collapse_expand_category(&category_id, &mappings, &categories, &feeds);
            }
            gtk::Inhibit(true)
        });
    }

    pub fn add_feed(&mut self, feed: &FeedModel, parent: &CategoryID) {
        let level = self.calculate_level(parent);
        let feed_widget = FeedRow::new(feed, level);
        let pos = self.calculate_position(feed.sort_index, parent);
        self.widget.insert(&feed_widget.borrow().row(), pos);
        {
            self.feeds.borrow_mut().insert(feed.feed_id.clone(), feed_widget);
        }

        {
            if let Some(feed_vec) = self.mappings.borrow_mut().get_mut(parent) {
                if !feed_vec.contains(&feed.feed_id) {
                    feed_vec.push(feed.feed_id.clone());
                }
                return
            }
        }
        
        {
            self.mappings.borrow_mut().insert(parent.clone(), vec![feed.feed_id.clone()]);
        }
    }

    fn collapse_expand_category(
        id: &CategoryID,
        mappings: &HandleMap<CategoryID, Vec<FeedID>>,
        categories: &HandleMap<CategoryID, Handle<CategoryRow>>,
        feeds: &HandleMap<FeedID, Handle<FeedRow>>) {

        let subcategories = Self::get_subcategories(id, categories);

        if let Some(category_handle) = categories.borrow().get(id) {
            let expanded = category_handle.borrow().is_expaneded();
            Self::collapse_expand_feeds(id, expanded, mappings, feeds);
            if let Some(subcategories) = subcategories {
                for subcategory in subcategories {
                    Self::collapse_expand_subcategory(&subcategory, expanded, mappings, categories, feeds);
                }
            }
        }
    }

    fn collapse_expand_subcategory(
        id: &CategoryID,
        expand: bool,
        mappings: &HandleMap<CategoryID, Vec<FeedID>>,
        categories: &HandleMap<CategoryID, Handle<CategoryRow>>,
        feeds: &HandleMap<FeedID, Handle<FeedRow>>) {

        let subcategories = Self::get_subcategories(id, categories);
        
        if let Some(category_handle) = categories.borrow().get(id) {
            if expand {
                category_handle.borrow().expand();
            }
            else {
                category_handle.borrow().collapse();
            }
            Self::collapse_expand_feeds(id, expand, mappings, feeds);
            if let Some(subcategories) = subcategories {
                for subcategory in subcategories {
                    Self::collapse_expand_subcategory(&subcategory, expand, mappings, categories, feeds);
                }
            }
        }
    }

    fn collapse_expand_feeds(
        id: &CategoryID,
        expand: bool,
        mappings: &HandleMap<CategoryID, Vec<FeedID>>,
        feeds: &HandleMap<FeedID, Handle<FeedRow>>) {

        if let Some(feed_id_vec) = mappings.borrow().get(id) {
            for feed_id in feed_id_vec {
                if let Some(feed_handle) = feeds.borrow().get(feed_id) {
                    if expand {
                        feed_handle.borrow().expand();
                    }
                    else {
                        feed_handle.borrow().collapse();
                    }
                }
            }
        }
    }

    fn get_subcategories(id: &CategoryID, categories: &HandleMap<CategoryID, Handle<CategoryRow>>) -> Option<Vec<CategoryID>> {
        let mut vec : Vec<CategoryID> = Vec::new();
        let borrowed_categories = categories.borrow();

        for item in borrowed_categories.values() {
            let category = item.borrow();
            if &category.parent == id {
                vec.push(category.id.clone());
            }
        }
        
        if vec.len() == 0 {
            return None
        }

        Some(vec)
    }

    fn calculate_level(&self, parent_id: &CategoryID) -> i32 {
        if parent_id == &NEWSFLASH_TOPLEVEL.clone() {
            return 0
        }
        if let Some(handle) = self.categories.borrow().get(&parent_id) {
            return 1 + self.calculate_level(&handle.borrow().parent)
        }
        0
    }

    fn calculate_position(&self, sort_index: Option<i32>, parent_id: &CategoryID) -> i32 {
        let mappings = self.mappings.borrow();
        let categories = self.categories.borrow();
        let feeds = self.feeds.borrow();
        let sibling_categories : Option<Vec<(CategoryID, Option<i32>)>> = match Self::get_subcategories(parent_id, &self.categories) {
            Some(sibling_categories) => {
                Some(sibling_categories.into_iter().map(|category_id| {
                    if let Some(category) = categories.get(&category_id) {
                        return (category_id, category.borrow().sort_index())
                    }
                    (category_id, None)
                }).collect())
            },
            None => None,
        };
        let sibling_feeds : Option<Vec<(FeedID, Option<i32>)>> = match mappings.get(parent_id) {
            Some(sibling_feeds) => {
                Some(sibling_feeds.into_iter().map(|feed_id| {
                    if let Some(feed) = feeds.get(&feed_id) {
                        return (feed_id.clone(), feed.borrow().sort_index())
                    }
                    (feed_id.clone(), None)
                }).collect())
            },
            None => None,
        };

        -1
    }
}