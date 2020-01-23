mod content_header;
mod error;
mod header_selection;

pub use self::content_header::ContentHeader;
pub use self::header_selection::HeaderSelection;

use self::error::{ContentPageError, ContentPageErrorKind};
use crate::app::Action;
use crate::article_list::{ArticleList, ArticleListArticleModel, ArticleListModel};
use crate::article_view::ArticleView;
use crate::main_window_state::MainWindowState;
use crate::settings::Settings;
use crate::sidebar::models::SidebarSelection;
use crate::sidebar::{FeedListTree, SideBar, TagListModel};
use crate::undo_bar::{UndoActionModel, UndoBar};
use crate::util::{BuilderHelper, Util, CHANNEL_ERROR};
use failure::ResultExt;
use futures::executor::ThreadPool;
use futures::channel::oneshot;
use glib::{Sender, futures::FutureExt};
use gtk::{Box, BoxExt, Button, WidgetExt};
use libhandy::Leaflet;
use news_flash::models::{Article, ArticleFilter, FatArticle, Feed, Marked, PluginCapabilities, PluginID, Read, NEWSFLASH_TOPLEVEL};
use news_flash::NewsFlash;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct ContentPage {
    sidebar: SideBar,
    article_list: Arc<RwLock<ArticleList>>,
    article_view: ArticleView,
    settings: Arc<RwLock<Settings>>,
}

impl ContentPage {
    pub fn new(builder: &BuilderHelper, settings: &Arc<RwLock<Settings>>, sender: Sender<Action>) -> Self {
        let feed_list_box = builder.get::<Box>("feedlist_box");
        let article_list_box = builder.get::<Box>("articlelist_box");
        let articleview_box = builder.get::<Box>("articleview_box");

        // workaround
        let minor_leaflet = builder.get::<Leaflet>("minor_leaflet");
        minor_leaflet.set_hexpand(false);

        let sidebar = SideBar::new(sender.clone());
        let article_list = Arc::new(RwLock::new(ArticleList::new(settings, sender.clone())));
        let article_view = ArticleView::new(settings);

        feed_list_box.pack_start(&sidebar.widget(), false, true, 0);
        article_list_box.pack_start(&article_list.read().widget(), false, true, 0);
        articleview_box.pack_start(&article_view.widget(), false, true, 0);

        let settings = settings.clone();

        ContentPage {
            sidebar,
            article_list,
            article_view,
            settings,
        }
    }

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), ContentPageError> {
        self.sidebar
            .set_service(id, user_name)
            .context(ContentPageErrorKind::SidebarService)?;
        Ok(())
    }

    pub fn update_article_list(
        &self,
        news_flash: &Arc<RwLock<Option<NewsFlash>>>,
        window_state: &Arc<RwLock<MainWindowState>>,
        undo_bar: &UndoBar,
        thread_pool: ThreadPool,
    ) -> Result<(), ContentPageError> {

        let (sender, receiver) = oneshot::channel::<Result<ArticleListModel, ContentPageErrorKind>>();

        let relevant_articles_loaded = self.article_list
            .read()
            .get_relevant_article_count(window_state.read().get_header_selection());

        let news_flash = news_flash.clone();
        let window_state_clone = window_state.clone();
        let current_undo_action = undo_bar.get_current_action();
        let settings = self.settings.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let limit = if window_state_clone.write().reset_article_list() {
                    MainWindowState::page_size()
                } else if relevant_articles_loaded as i64 >= MainWindowState::page_size() {
                    relevant_articles_loaded as i64
                } else {
                    MainWindowState::page_size()
                };
                let mut list_model = ArticleListModel::new(&settings.read().get_article_list_order());
                let mut articles = match Self::load_articles(news_flash, &window_state_clone, &settings, &current_undo_action, limit, None) {
                    Ok(articles) => articles,
                    Err(error) => {
                        sender.send(Err(error.kind())).expect(CHANNEL_ERROR);
                        return;
                    },
                };
    
                let feeds = match news_flash.get_feeds() {
                    Ok((feeds, _)) => feeds,
                    Err(_error) => {
                        sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                        return;
                    },
                };
                let _: Vec<Result<(), ContentPageError>> = articles
                    .drain(..)
                    .map(|article| {
                        let feed = feeds
                            .iter()
                            .find(|&f| f.feed_id == article.feed_id)
                            .ok_or_else(|| ContentPageErrorKind::FeedTitle)?;
                        list_model
                            .add(article, &feed)
                            .context(ContentPageErrorKind::ArticleList)?;
                        Ok(())
                    })
                    .collect();

                sender.send(Ok(list_model)).expect(CHANNEL_ERROR);
            }
        };

        let window_state_clone = window_state.clone();
        let article_list = self.article_list.clone();
        let glib_future = receiver.map(move |res| {
            if let Ok(res) = res {
                if let Ok(article_list_model) = res {
                    article_list.write().update(article_list_model, &window_state_clone);
                }
            }
        });

        thread_pool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);

        Ok(())
    }

    pub fn load_more_articles(
        &mut self,
        news_flash_handle: &Arc<RwLock<Option<NewsFlash>>>,
        window_state: &Arc<RwLock<MainWindowState>>,
        undo_bar: &UndoBar,
        thread_pool: ThreadPool,
    ) -> Result<(), ContentPageError> {

        let (sender, receiver) = oneshot::channel::<Result<ArticleListModel, ContentPageErrorKind>>();

        let relevant_articles_loaded = self
            .article_list
            .read()
            .get_relevant_article_count(window_state.read().get_header_selection());
        
        let current_undo_action = undo_bar.get_current_action();
        let settings = self.settings.clone();
        let news_flash = news_flash_handle.clone();
        let window_state = window_state.clone();
        let thread_future = async move {
            let mut list_model = ArticleListModel::new(&settings.read().get_article_list_order());
            
            if let Some(news_flash) = news_flash.read().as_ref() {
                let mut articles = match Self::load_articles(
                    news_flash,
                    &window_state,
                    &settings,
                    &current_undo_action,
                    MainWindowState::page_size(),
                    Some(relevant_articles_loaded as i64),
                ) {
                    Ok(articles) => articles,
                    Err(error) => {
                        sender.send(Err(error.kind())).expect(CHANNEL_ERROR);
                        return;
                    },
                };
                let feeds = match news_flash.get_feeds() {
                    Ok((feeds, _)) => feeds,
                    Err(_error) => {
                        sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                        return;
                    },
                };
                let _: Vec<Result<(), ContentPageError>> = articles
                    .drain(..)
                    .map(|article| {
                        let feed = feeds
                            .iter()
                            .find(|&f| f.feed_id == article.feed_id)
                            .ok_or_else(|| ContentPageErrorKind::FeedTitle)?;
                        list_model
                            .add(article, &feed)
                            .context(ContentPageErrorKind::ArticleList)?;
                        Ok(())
                    })
                    .collect();
                
                sender.send(Ok(list_model)).expect(CHANNEL_ERROR);
            }
        };

        let article_list = self.article_list.clone();
        let glib_future = receiver.map(move |res| {
            if let Ok(res) = res {
                if let Ok(article_list_model) = res {
                    article_list.write().add_more_articles(article_list_model);
                }
            }
        });

        thread_pool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);

        Ok(())
    }

    fn load_articles(
        news_flash: &NewsFlash,
        window_state: &RwLock<MainWindowState>,
        settings: &Arc<RwLock<Settings>>,
        current_undo_action: &Option<UndoActionModel>,
        limit: i64,
        offset: Option<i64>,
    ) -> Result<Vec<Article>, ContentPageError> {
        let unread = match window_state.read().get_header_selection() {
            HeaderSelection::All | HeaderSelection::Marked => None,
            HeaderSelection::Unread => Some(Read::Unread),
        };
        let marked = match window_state.read().get_header_selection() {
            HeaderSelection::All | HeaderSelection::Unread => None,
            HeaderSelection::Marked => Some(Marked::Marked),
        };
        let feed = match &window_state.read().get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Cateogry(_) | SidebarSelection::Tag(_) => None,
            SidebarSelection::Feed((id, _title)) => Some(id.clone()),
        };
        let category = match &window_state.read().get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_) | SidebarSelection::Tag(_) => None,
            SidebarSelection::Cateogry((id, _title)) => Some(id.clone()),
        };
        let tag = match &window_state.read().get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_) | SidebarSelection::Cateogry(_) => None,
            SidebarSelection::Tag((id, _title)) => Some(id.clone()),
        };
        let search_term = window_state.read().get_search_term().clone();
        let (feed_blacklist, category_blacklist) = match current_undo_action {
            Some(action) => match action {
                UndoActionModel::DeleteFeed((feed_id, _label)) => (Some(vec![feed_id.clone()]), None),
                UndoActionModel::DeleteCategory((category_id, _label)) => (None, Some(vec![category_id.clone()])),
                UndoActionModel::DeleteTag((_tag_id, _label)) => (None, None),
            },
            None => (None, None),
        };

        let articles = news_flash
            .get_articles(ArticleFilter {
                limit: Some(limit),
                offset,
                order: Some(settings.read().get_article_list_order()),
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
                search_term,
            })
            .context(ContentPageErrorKind::DataBase)?;

        Ok(articles)
    }

    pub fn update_sidebar(
        &mut self,
        news_flash: &RwLock<Option<NewsFlash>>,
        state: &RwLock<MainWindowState>,
        undo_bar: &UndoBar,
    ) -> Result<(), ContentPageError> {
        if let Some(news_flash) = news_flash.read().as_ref() {
            let mut tree = FeedListTree::new();
            let categories = news_flash.get_categories().context(ContentPageErrorKind::DataBase)?;
            let (feeds, mappings) = news_flash.get_feeds().context(ContentPageErrorKind::DataBase)?;

            // collect unread and marked counts
            let feed_count_map = match state.read().get_header_selection() {
                HeaderSelection::All | HeaderSelection::Unread => news_flash
                    .unread_count_feed_map()
                    .context(ContentPageErrorKind::DataBase)?,
                HeaderSelection::Marked => news_flash
                    .marked_count_feed_map()
                    .context(ContentPageErrorKind::DataBase)?,
            };

            let pending_delete_category = undo_bar.get_current_action().map(|a| {
                if let UndoActionModel::DeleteCategory((id, _label)) = a {
                    Some(id)
                } else {
                    None
                }
            }).flatten();
            let pending_delete_feed = undo_bar.get_current_action().map(|a| {
                if let UndoActionModel::DeleteFeed((id, _label)) = a {
                    Some(id)
                } else {
                    None
                }
            }).flatten();

            // feedlist: Categories
            for category in &categories {
                if let Some(pending_delete_category) = &pending_delete_category {
                    if pending_delete_category == &category.category_id {
                        continue;
                    }
                }
                

                let category_item_count = Util::calculate_item_count_for_category(
                    &category.category_id,
                    &categories,
                    &mappings,
                    &feed_count_map,
                    &pending_delete_feed,
                    &pending_delete_category,
                );

                tree.add_category(category, category_item_count)
                    .context(ContentPageErrorKind::SidebarModels)?
            }

            // feedlist: Feeds
            for mapping in &mappings {
                if let Some(undo_action) = undo_bar.get_current_action() {
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

                let feed = feeds
                    .iter()
                    .find(|feed| feed.feed_id == mapping.feed_id)
                    .ok_or_else(|| ContentPageErrorKind::FeedTitle)?;

                let item_count = match feed_count_map.get(&mapping.feed_id) {
                    Some(count) => *count,
                    None => 0,
                };
                tree.add_feed(&feed, &mapping, item_count)
                    .context(ContentPageErrorKind::SidebarModels)?
            }

            // tag list
            let plugin_features = news_flash.features().context(ContentPageErrorKind::NewsFlashFeatures)?;
            let support_tags = plugin_features.contains(PluginCapabilities::SUPPORT_TAGS);

            if !support_tags {
                self.sidebar.hide_taglist();
            } else {
                let mut list = TagListModel::new();
                let tags = news_flash.get_tags().context(ContentPageErrorKind::DataBase)?;

                if tags.is_empty() {
                    self.sidebar.hide_taglist();
                } else {
                    for tag in tags {
                        if let Some(UndoActionModel::DeleteTag((id, _label))) = undo_bar.get_current_action() {
                            if id == tag.tag_id {
                                continue;
                            }
                        }
                        list.add(&tag).context(ContentPageErrorKind::SidebarModels)?
                    }

                    self.sidebar.update_taglist(list);
                    self.sidebar.show_taglist();
                }
            }

            //let total_item_count = feed_count_map.iter().map(|(_key, value)| value).sum();
            let total_item_count = Util::calculate_item_count_for_category(
                &NEWSFLASH_TOPLEVEL,
                &categories,
                &mappings,
                &feed_count_map,
                &pending_delete_feed,
                &pending_delete_category,
            );

            self.sidebar.update_feedlist(tree);
            self.sidebar.update_all(total_item_count);

            return Ok(());
        }

        Err(ContentPageErrorKind::NewsFlashHandle.into())
    }

    pub fn article_view_show(&self, article: FatArticle, feed: &Feed) {
        self.article_view.show_article(article, feed.label.clone());
    }

    pub fn article_view_scroll_diff(&self, diff: f64) -> Result<(), ContentPageError> {
        self.article_view
            .animate_scroll_diff(diff)
            .context(ContentPageErrorKind::ArticleView)?;
        Ok(())
    }

    pub fn article_view_close(&self) {
        self.article_view.close_article()
    }

    pub fn article_view_visible_article(&self) -> Option<FatArticle> {
        self.article_view.get_visible_article()
    }

    pub fn article_view_update_visible_article(&self, read: Option<Read>, marked: Option<Marked>) {
        self.article_view.update_visible_article(read, marked);
    }

    pub fn article_view_redraw(&self) {
        self.article_view.redraw_article()
    }

    pub fn select_next_article(&self) {
        self.article_list.read().select_next_article()
    }

    pub fn select_prev_article(&self) {
        self.article_list.read().select_prev_article()
    }

    pub fn get_selected_article_model(&self) -> Option<ArticleListArticleModel> {
        self.article_list.read().get_selected_article_model()
    }

    pub fn sidebar_select_next_item(&self) -> Result<(), ContentPageError> {
        self.sidebar
            .select_next_item()
            .context(ContentPageErrorKind::SidebarSelection)?;
        Ok(())
    }

    pub fn sidebar_select_prev_item(&self) -> Result<(), ContentPageError> {
        self.sidebar
            .select_prev_item()
            .context(ContentPageErrorKind::SidebarSelection)?;
        Ok(())
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
