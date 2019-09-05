mod content_header;
mod header_selection;

pub use self::content_header::ContentHeader;
pub use self::header_selection::HeaderSelection;

use crate::article_list::{ArticleList, ArticleListArticleModel, ArticleListModel};
use crate::article_view::ArticleView;
use crate::main_window_state::MainWindowState;
use crate::settings::Settings;
use crate::sidebar::models::SidebarSelection;
use crate::sidebar::{FeedListCountType, FeedListTree, SideBar, TagListModel};
use crate::undo_bar::{UndoActionModel, UndoBar};
use crate::util::{BuilderHelper, GtkHandle, Util};
use failure::format_err;
use failure::Error;
use gtk::{Box, BoxExt, Button, WidgetExt};
use libhandy::Leaflet;
use news_flash::models::{
    Article, ArticleFilter, ArticleID, CategoryID, FeedID, Marked, PluginCapabilities, PluginID, Read,
};
use news_flash::NewsFlash;

pub struct ContentPage {
    sidebar: SideBar,
    article_list: ArticleList,
    article_view: ArticleView,
    settings: GtkHandle<Settings>,
}

impl ContentPage {
    pub fn new(builder: &BuilderHelper, settings: &GtkHandle<Settings>) -> Result<Self, Error> {
        let feed_list_box = builder.get::<Box>("feedlist_box");
        let article_list_box = builder.get::<Box>("articlelist_box");
        let articleview_box = builder.get::<Box>("articleview_box");

        // workaround
        let minor_leaflet = builder.get::<Leaflet>("minor_leaflet");
        minor_leaflet.set_hexpand(false);

        let sidebar = SideBar::new()?;
        let article_list = ArticleList::new(settings)?;
        let article_view = ArticleView::new(settings)?;

        feed_list_box.pack_start(&sidebar.widget(), false, true, 0);
        article_list_box.pack_start(&article_list.widget(), false, true, 0);
        articleview_box.pack_start(&article_view.widget(), false, true, 0);

        let settings = settings.clone();

        Ok(ContentPage {
            sidebar,
            article_list,
            article_view,
            settings,
        })
    }

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), Error> {
        self.sidebar.set_service(id, user_name)?;
        Ok(())
    }

    pub fn update_article_list(
        &mut self,
        news_flash_handle: &GtkHandle<Option<NewsFlash>>,
        window_state: &GtkHandle<MainWindowState>,
        undo_bar: &GtkHandle<UndoBar>,
    ) {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            self.update_article_list_from_ref(news_flash, &mut *window_state.borrow_mut(), undo_bar);
        }
    }

    fn update_article_list_from_ref(
        &mut self,
        news_flash: &mut NewsFlash,
        window_state: &mut MainWindowState,
        undo_bar: &GtkHandle<UndoBar>,
    ) {
        let relevant_articles_loaded = self
            .article_list
            .get_relevant_article_count(window_state.get_header_selection());
        let limit = if window_state.reset_article_list() {
            MainWindowState::page_size()
        } else if relevant_articles_loaded as i64 >= MainWindowState::page_size() {
            relevant_articles_loaded as i64
        } else {
            MainWindowState::page_size()
        };
        let mut list_model = ArticleListModel::new(&self.settings.borrow().get_article_list_order());
        let mut articles = Self::load_articles(news_flash, &window_state, &self.settings, undo_bar, limit, None);

        let (feeds, _) = news_flash.get_feeds().unwrap();
        let _: Vec<_> = articles
            .drain(..)
            .map(|article| {
                let feed = feeds.iter().find(|&f| f.feed_id == article.feed_id).unwrap();
                let favicon = match news_flash.get_icon_info(&feed) {
                    Ok(favicon) => Some(favicon),
                    Err(_) => None,
                };
                list_model.add(article, feed.label.clone(), favicon)
            })
            .collect();
        self.article_list.update(list_model, window_state);
    }

    pub fn load_more_articles(
        &mut self,
        news_flash_handle: &GtkHandle<Option<NewsFlash>>,
        window_state: &GtkHandle<MainWindowState>,
        undo_bar: &GtkHandle<UndoBar>,
    ) -> Result<(), Error> {
        let window_state = window_state.borrow().clone();
        let relevant_articles_loaded = self
            .article_list
            .get_relevant_article_count(window_state.get_header_selection());
        let mut list_model = ArticleListModel::new(&self.settings.borrow().get_article_list_order());
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            let mut articles = Self::load_articles(
                news_flash,
                &window_state,
                &self.settings,
                undo_bar,
                MainWindowState::page_size(),
                Some(relevant_articles_loaded as i64),
            );
            let (feeds, _) = news_flash.get_feeds().unwrap();
            let _: Vec<_> = articles
                .drain(..)
                .map(|article| {
                    let feed = feeds.iter().find(|&f| f.feed_id == article.feed_id).unwrap();
                    let favicon = match news_flash.get_icon_info(&feed) {
                        Ok(favicon) => Some(favicon),
                        Err(_) => None,
                    };
                    list_model.add(article, feed.label.clone(), favicon)
                })
                .collect();
            self.article_list.add_more_articles(list_model)?;
        }
        Ok(())
    }

    fn load_articles(
        news_flash: &mut NewsFlash,
        window_state: &MainWindowState,
        settings: &GtkHandle<Settings>,
        undo_bar: &GtkHandle<UndoBar>,
        limit: i64,
        offset: Option<i64>,
    ) -> Vec<Article> {
        let unread = match window_state.get_header_selection() {
            HeaderSelection::All | HeaderSelection::Marked => None,
            HeaderSelection::Unread => Some(Read::Unread),
        };
        let marked = match window_state.get_header_selection() {
            HeaderSelection::All | HeaderSelection::Unread => None,
            HeaderSelection::Marked => Some(Marked::Marked),
        };
        let feed = match &window_state.get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Cateogry(_) | SidebarSelection::Tag(_) => None,
            SidebarSelection::Feed((id, _title)) => Some(id.clone()),
        };
        let category = match &window_state.get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_) | SidebarSelection::Tag(_) => None,
            SidebarSelection::Cateogry((id, _title)) => Some(id.clone()),
        };
        let tag = match &window_state.get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_) | SidebarSelection::Cateogry(_) => None,
            SidebarSelection::Tag((id, _title)) => Some(id.clone()),
        };
        let (feed_blacklist, category_blacklist) = match undo_bar.borrow().get_current_action() {
            Some(action) => match action {
                UndoActionModel::DeleteFeed((feed_id, _label)) => (Some(vec![feed_id]), None),
                UndoActionModel::DeleteCategory((category_id, _label)) => (None, Some(vec![category_id])),
                UndoActionModel::DeleteTag((_tag_id, _label)) => (None, None),
            },
            None => (None, None),
        };

        news_flash
            .get_articles(ArticleFilter {
                limit: Some(limit),
                offset,
                order: Some(settings.borrow().get_article_list_order()),
                unread,
                marked,
                feed,
                feed_blacklist,
                category,
                category_blacklist,
                tag,
                ids: None,
                newer_than: None,
                older_than: None,
            })
            .unwrap()
    }

    pub fn update_sidebar(
        &mut self,
        news_flash_handle: &GtkHandle<Option<NewsFlash>>,
        state: &GtkHandle<MainWindowState>,
        undo_bar: &GtkHandle<UndoBar>,
    ) {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            let now = std::time::Instant::now();

            let feedlist_count_type = match state.borrow().get_header_selection() {
                HeaderSelection::All | HeaderSelection::Unread => FeedListCountType::Unread,
                HeaderSelection::Marked => FeedListCountType::Marked,
            };

            let mut tree = FeedListTree::new(&feedlist_count_type);
            let categories = news_flash.get_categories().unwrap();
            let (feeds, mappings) = news_flash.get_feeds().unwrap();

            

            // collect unread and marked counts
            let feed_unread_counts = news_flash.unread_count_feed_map().unwrap();
            let feed_marked_counts = news_flash.marked_count_feed_map().unwrap();

            // feedlist: Categories
            for category in &categories {
                if let Some(UndoActionModel::DeleteCategory((id, _label))) = undo_bar.borrow().get_current_action() {
                    if id == category.category_id {
                        continue;
                    }
                }

                let unread_count = Util::calculate_item_count_for_category(
                    &category.category_id,
                    &categories,
                    &mappings,
                    &feed_unread_counts,
                );
                let marked_count = Util::calculate_item_count_for_category(
                    &category.category_id,
                    &categories,
                    &mappings,
                    &feed_marked_counts,
                );

                tree.add_category(category, unread_count as i32, marked_count as i32)
                    .unwrap();
            }

            // feedlist: Feeds
            for mapping in &mappings {
                if let Some(undo_action) = undo_bar.borrow().get_current_action() {
                    match undo_action {
                        UndoActionModel::DeleteFeed((id, _label)) => {
                            if id == mapping.feed_id {
                                continue;
                            }
                        }
                        UndoActionModel::DeleteCategory((id, _label)) => {
                            if id == mapping.category_id {
                                continue;
                            }
                        }
                        _ => {}
                    }
                }

                let feed = feeds.iter().find(|feed| feed.feed_id == mapping.feed_id).unwrap();

                let unread_count = match feed_unread_counts.get(&mapping.feed_id) {
                    Some(count) => *count,
                    None => 0,
                };
                let marked_count = match feed_marked_counts.get(&mapping.feed_id) {
                    Some(count) => *count,
                    None => 0,
                };
                let favicon = match news_flash.get_icon_info(&feed) {
                    Ok(favicon) => Some(favicon),
                    Err(_) => None,
                };
                tree.add_feed(&feed, &mapping, unread_count as i32, marked_count as i32, favicon)
                    .unwrap();
            }

            // tag list
            let plugin_features = news_flash.features().unwrap();
            let support_tags = plugin_features.contains(PluginCapabilities::SUPPORT_TAGS);

            if !support_tags {
                self.sidebar.hide_taglist();
            } else {
                let mut list = TagListModel::new();
                let tags = news_flash.get_tags().unwrap();

                if tags.is_empty() {
                    self.sidebar.hide_taglist();
                } else {
                    for tag in tags {
                        if let Some(UndoActionModel::DeleteTag((id, _label))) = undo_bar.borrow().get_current_action() {
                            if id == tag.tag_id {
                                continue;
                            }
                        }
                        list.add(&tag).unwrap();
                    }

                    self.sidebar.update_taglist(list);
                    self.sidebar.show_taglist();
                }
            }

            let total_unread_count = feed_unread_counts.iter().map(|(_key, value)| value).sum();
            let total_marked_count = feed_marked_counts.iter().map(|(_key, value)| value).sum();

            println!("db read: {} us", now.elapsed().as_micros());

            self.sidebar.update_feedlist(tree);
            self.sidebar.update_all(total_unread_count, total_marked_count);
        }
    }

    pub fn sidebar_change_count_type(&mut self, new_type: &FeedListCountType) {
        let now = std::time::Instant::now();
        let tree = self.sidebar.clone_feedlist_tree_with_new_count_type(new_type);
        println!("switch count: {} us", now.elapsed().as_micros());

        let now = std::time::Instant::now();
        self.sidebar.update_feedlist(tree);
        self.sidebar.update_all_label();
        println!("UI update: {} us", now.elapsed().as_micros());
    }

    pub fn sidebar_decrease_feed_count(&mut self, id: &FeedID, count_type: &FeedListCountType) {
        let mut tree = self.sidebar.clone_feedlist_tree();
        tree.feed_decrease_count(id, count_type);

        let total_count = self.sidebar.get_unread_all_for_type(count_type) - 1;

        self.sidebar.update_feedlist(tree);
        self.sidebar.update_all_for_type(total_count, count_type);
    }

    pub fn sidebar_increase_feed_count(&mut self, id: &FeedID, count_type: &FeedListCountType) {
        let mut tree = self.sidebar.clone_feedlist_tree();
        tree.feed_increase_count(id, count_type);

        let total_count = self.sidebar.get_unread_all_for_type(count_type) + 1;

        self.sidebar.update_feedlist(tree);
        self.sidebar.update_all_for_type(total_count, count_type);
    }

    pub fn sidebar_reset_feed_count(&mut self, id: &FeedID, count_type: &FeedListCountType) {
        let mut tree = self.sidebar.clone_feedlist_tree();
        let old_count = tree.feed_reset_count(id, count_type) as i64;

        let total_count = self.sidebar.get_unread_all_for_type(count_type) - old_count;

        self.sidebar.update_feedlist(tree);
        self.sidebar.update_all_for_type(total_count, count_type);
    }

    pub fn sidebar_reset_category_count(&mut self, id: &CategoryID, count_type: &FeedListCountType) {
        let mut tree = self.sidebar.clone_feedlist_tree();
        let old_count = tree.category_reset_count(id, count_type) as i64;

        let total_count = self.sidebar.get_unread_all_for_type(count_type) - old_count;

        if &self.sidebar.get_count_type() != count_type {
            self.sidebar.update_feedlist(tree);
        }
        self.sidebar.update_all_for_type(total_count, count_type);
    }

    pub fn sidebar_reset_all_count(&mut self, count_type: &FeedListCountType) {
        let mut tree = self.sidebar.clone_feedlist_tree();
        tree.all_reset_count(count_type);

        // FIXME: only update if necessary
        self.sidebar.update_feedlist(tree);
        self.sidebar.update_all_for_type(0, count_type);
    }

    pub fn article_view_show(
        &mut self,
        article_id: &ArticleID,
        news_flash_handle: &GtkHandle<Option<NewsFlash>>,
    ) -> Result<(), Error> {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            let article = news_flash.get_fat_article(article_id).unwrap();
            let (feeds, _) = news_flash.get_feeds().unwrap();
            let feed = feeds.iter().find(|&f| f.feed_id == article.feed_id).unwrap();
            self.article_view.show_article(article, feed.label.clone())?;
            return Ok(());
        }
        Err(format_err!("some err"))
    }

    pub fn article_view_scroll_diff(&self, diff: f64) -> Result<(), Error> {
        self.article_view.animate_scroll_diff(diff)
    }

    pub fn article_view_close(&self) -> Result<(), Error> {
        self.article_view.close_article()
    }

    pub fn article_view_redraw(&mut self) -> Result<(), Error> {
        self.article_view.redraw_article()
    }

    pub fn select_next_article(&self) {
        self.article_list.select_next_article()
    }

    pub fn select_prev_article(&self) {
        self.article_list.select_prev_article()
    }

    pub fn get_selected_article_model(&self) -> Option<ArticleListArticleModel> {
        self.article_list.get_selected_article_model()
    }

    pub fn sidebar_select_next_item(&self) -> Result<(), Error> {
        self.sidebar.select_next_item()
    }

    pub fn sidebar_select_prev_item(&self) -> Result<(), Error> {
        self.sidebar.select_prev_item()
    }

    pub fn sidebar_expand_collase_category(&self) {
        self.sidebar.expand_collapse_selected_category()
    }

    pub fn sidebar_get_selection(&self) -> SidebarSelection {
        self.sidebar.get_selection()
    }

    pub fn sidebar_select_all_button_no_update(&self) {
        self.sidebar.select_all_button_no_update();
    }

    pub fn sidebar_get_add_button(&self) -> Button {
        self.sidebar.get_add_button()
    }
}
