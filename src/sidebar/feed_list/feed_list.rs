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
};
use sidebar::{
    Category,
    Feed,
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
    categories: HandleMap<CategoryID, Handle<Category>>,
    feeds: HandleMap<FeedID, Handle<Feed>>,
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
        let category_widget = Category::new(category);
        let mappings = self.mappings.clone();
        let feeds = self.feeds.clone();
        let categories = self.categories.clone();
        let category_id = category.category_id.clone();
        self.widget.insert(&category_widget.borrow().row(), -1);
        {
            categories.borrow_mut().insert(category.category_id.clone(), category_widget.clone());
        }
        category_widget.borrow().expander_event().connect_button_press_event(move |_widget, event| {
            if event.get_event_type() == EventType::ButtonPress {
                if let Some(category_handle) = categories.borrow().get(&category_id) {
                    let expaneded = category_handle.borrow().is_expaneded();
                    if let Some(feed_id_vec) = mappings.borrow().get(&category_id) {
                        for feed_id in feed_id_vec {
                            if let Some(feed_handle) = feeds.borrow().get(feed_id) {
                                // expanded has been set already, now only apply to feeds
                                if expaneded {
                                    feed_handle.borrow().expand();
                                }
                                else {
                                    feed_handle.borrow().collapse();
                                }
                            }
                        }
                    }
                }
            }
            gtk::Inhibit(true)
        });
    }

    pub fn add_feed(&mut self, feed: &FeedModel, parent: &CategoryID) {
        let feed_widget = Feed::new(feed);
        self.widget.insert(&feed_widget.borrow().row(), -1);
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
}