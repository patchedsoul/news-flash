mod content_header;
mod error;
mod header_selection;

pub use self::content_header::ContentHeader;
pub use self::header_selection::HeaderSelection;

use self::error::{ContentPageError, ContentPageErrorKind};
use crate::app::Action;
use crate::article_list::{ArticleList, ArticleListModel};
use crate::article_view::ArticleView;
use crate::main_window_state::MainWindowState;
use crate::settings::Settings;
use crate::sidebar::models::SidebarSelection;
use crate::sidebar::{FeedListTree, SideBar, TagListModel};
use crate::undo_bar::{UndoActionModel, UndoBar};
use crate::util::{BuilderHelper, Util, CHANNEL_ERROR};
use failure::ResultExt;
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures_util::future::FutureExt;
use glib::Sender;
use gtk::{Box, BoxExt, WidgetExt};
use libhandy::Leaflet;
use news_flash::models::{Article, ArticleFilter, Marked, PluginCapabilities, PluginID, Read, NEWSFLASH_TOPLEVEL};
use news_flash::NewsFlash;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct ContentPage {
    pub sidebar: Arc<RwLock<SideBar>>,
    pub article_list: Arc<RwLock<ArticleList>>,
    pub article_view: ArticleView,
    settings: Arc<RwLock<Settings>>,
    state: Arc<RwLock<MainWindowState>>,
}

impl ContentPage {
    pub fn new(
        builder: &BuilderHelper,
        state: &Arc<RwLock<MainWindowState>>,
        settings: &Arc<RwLock<Settings>>,
        content_header: &Arc<ContentHeader>,
        sender: Sender<Action>,
    ) -> Self {
        let feed_list_box = builder.get::<Box>("feedlist_box");
        let article_list_box = builder.get::<Box>("articlelist_box");
        let articleview_box = builder.get::<Box>("articleview_box");

        // workaround
        let minor_leaflet = builder.get::<Leaflet>("minor_leaflet");
        minor_leaflet.set_hexpand(false);

        let sidebar = Arc::new(RwLock::new(SideBar::new(state, sender.clone())));
        let article_list = Arc::new(RwLock::new(ArticleList::new(
            settings,
            content_header,
            state,
            sender.clone(),
        )));
        let article_view = ArticleView::new(settings);

        feed_list_box.pack_start(&sidebar.read().widget(), false, true, 0);
        article_list_box.pack_start(&article_list.read().widget(), false, true, 0);
        articleview_box.pack_start(&article_view.widget(), false, true, 0);

        let settings = settings.clone();

        ContentPage {
            sidebar,
            article_list,
            article_view,
            settings,
            state: state.clone(),
        }
    }

    pub fn clear(&self) {
        self.article_view.close_article();

        let list_model = ArticleListModel::new(&self.settings.read().get_article_list_order());
        self.article_list.write().update(list_model, &self.state);

        let feed_tree_model = FeedListTree::new();
        self.sidebar.write().update_feedlist(feed_tree_model);

        let tag_list_model = TagListModel::new();
        self.sidebar.write().update_taglist(tag_list_model);
        self.sidebar.read().hide_taglist();

        self.sidebar
            .read()
            .set_service(None, None)
            .expect("Resetting Logo & Username should always work");
    }

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), ContentPageError> {
        self.sidebar
            .read()
            .set_service(Some(id), user_name)
            .context(ContentPageErrorKind::SidebarService)?;
        Ok(())
    }

    pub fn update_article_list(
        &self,
        news_flash: &Arc<RwLock<Option<NewsFlash>>>,
        undo_bar: &UndoBar,
        thread_pool: ThreadPool,
    ) {
        let (sender, receiver) = oneshot::channel::<Result<ArticleListModel, ContentPageErrorKind>>();

        let relevant_articles_loaded = self
            .article_list
            .read()
            .get_relevant_article_count(self.state.read().get_header_selection());

        let news_flash = news_flash.clone();
        let window_state = self.state.clone();
        let current_undo_action = undo_bar.get_current_action();
        let settings = self.settings.clone();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let limit = if window_state.write().reset_article_list() {
                    MainWindowState::page_size()
                } else if relevant_articles_loaded as i64 >= MainWindowState::page_size() {
                    relevant_articles_loaded as i64
                } else {
                    MainWindowState::page_size()
                };
                let mut list_model = ArticleListModel::new(&settings.read().get_article_list_order());
                let mut articles = match Self::load_articles(
                    news_flash,
                    &window_state,
                    &settings,
                    &current_undo_action,
                    limit,
                    None,
                ) {
                    Ok(articles) => articles,
                    Err(error) => {
                        sender.send(Err(error.kind())).expect(CHANNEL_ERROR);
                        return;
                    }
                };

                let feeds = match news_flash.get_feeds() {
                    Ok((feeds, _)) => feeds,
                    Err(_error) => {
                        sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                        return;
                    }
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

        let window_state = self.state.clone();
        let article_list = self.article_list.clone();
        let glib_future = receiver.map(move |res| {
            if let Ok(res) = res {
                if let Ok(article_list_model) = res {
                    article_list.write().update(article_list_model, &window_state);
                }
            }
        });

        thread_pool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    pub fn load_more_articles(
        &self,
        news_flash_handle: &Arc<RwLock<Option<NewsFlash>>>,
        window_state: &Arc<RwLock<MainWindowState>>,
        undo_bar: &UndoBar,
        thread_pool: ThreadPool,
    ) {
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
                    }
                };
                let feeds = match news_flash.get_feeds() {
                    Ok((feeds, _)) => feeds,
                    Err(_error) => {
                        sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                        return;
                    }
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
        &self,
        news_flash: &Arc<RwLock<Option<NewsFlash>>>,
        undo_bar: &UndoBar,
        thread_pool: ThreadPool,
    ) {
        let (sender, receiver) =
            oneshot::channel::<Result<(i64, FeedListTree, Option<TagListModel>), ContentPageErrorKind>>();

        let news_flash = news_flash.clone();
        let state = self.state.clone();
        let current_undo_action = undo_bar.get_current_action();
        let thread_future = async move {
            if let Some(news_flash) = news_flash.read().as_ref() {
                let mut tree = FeedListTree::new();
                let mut tag_list_model: Option<TagListModel> = None;

                let categories = match news_flash.get_categories() {
                    Ok(categories) => categories,
                    Err(_error) => {
                        sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                        return;
                    }
                };
                let (feeds, mappings) = match news_flash.get_feeds() {
                    Ok(res) => res,
                    Err(_error) => {
                        sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                        return;
                    }
                };

                // collect unread and marked counts
                let feed_count_map = match state.read().get_header_selection() {
                    HeaderSelection::All | HeaderSelection::Unread => match news_flash.unread_count_feed_map() {
                        Ok(res) => res,
                        Err(_error) => {
                            sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                            return;
                        }
                    },
                    HeaderSelection::Marked => match news_flash.marked_count_feed_map() {
                        Ok(res) => res,
                        Err(_error) => {
                            sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                            return;
                        }
                    },
                };

                let pending_delete_category = current_undo_action
                    .clone()
                    .map(|a| {
                        if let UndoActionModel::DeleteCategory((id, _label)) = a {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .flatten();
                let pending_delete_feed = current_undo_action
                    .clone()
                    .map(|a| {
                        if let UndoActionModel::DeleteFeed((id, _label)) = a {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .flatten();

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

                    if tree.add_category(category, category_item_count).is_err() {
                        sender
                            .send(Err(ContentPageErrorKind::SidebarModels))
                            .expect(CHANNEL_ERROR);
                        return;
                    }
                }

                // feedlist: Feeds
                for mapping in &mappings {
                    if let Some(undo_action) = &current_undo_action {
                        match undo_action {
                            UndoActionModel::DeleteFeed((id, _label)) => {
                                if id == &mapping.feed_id {
                                    continue;
                                }
                            }
                            UndoActionModel::DeleteCategory((id, _label)) => {
                                if id == &mapping.category_id {
                                    continue;
                                }
                            }
                            _ => {}
                        }
                    }

                    let feed = match feeds.iter().find(|feed| feed.feed_id == mapping.feed_id) {
                        Some(res) => res,
                        None => {
                            sender.send(Err(ContentPageErrorKind::FeedTitle)).expect(CHANNEL_ERROR);
                            return;
                        }
                    };

                    let item_count = match feed_count_map.get(&mapping.feed_id) {
                        Some(count) => *count,
                        None => 0,
                    };
                    if tree.add_feed(&feed, &mapping, item_count).is_err() {
                        //sender.send(Err(ContentPageErrorKind::SidebarModels)).expect(CHANNEL_ERROR);
                    }
                }

                // tag list
                let plugin_features = match news_flash.features() {
                    Ok(features) => features,
                    Err(_error) => {
                        sender
                            .send(Err(ContentPageErrorKind::NewsFlashFeatures))
                            .expect(CHANNEL_ERROR);
                        return;
                    }
                };
                let support_tags = plugin_features.contains(PluginCapabilities::SUPPORT_TAGS);

                if support_tags {
                    let mut list = TagListModel::new();
                    let tags = match news_flash.get_tags() {
                        Ok(tags) => tags,
                        Err(_error) => {
                            sender.send(Err(ContentPageErrorKind::DataBase)).expect(CHANNEL_ERROR);
                            return;
                        }
                    };

                    if !tags.is_empty() {
                        for tag in tags {
                            if let Some(UndoActionModel::DeleteTag((id, _label))) = current_undo_action.clone() {
                                if id == tag.tag_id {
                                    continue;
                                }
                            }
                            if list.add(&tag).is_err() {
                                sender
                                    .send(Err(ContentPageErrorKind::SidebarModels))
                                    .expect(CHANNEL_ERROR);
                                return;
                            }
                        }
                    }

                    tag_list_model = Some(list);
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

                sender
                    .send(Ok((total_item_count, tree, tag_list_model)))
                    .expect(CHANNEL_ERROR);
            }
        };

        let sidebar = self.sidebar.clone();
        let glib_future = receiver.map(move |res| {
            if let Ok(res) = res {
                if let Ok((total_count, feed_list_model, tag_list_model)) = res {
                    sidebar.write().update_feedlist(feed_list_model);
                    sidebar.write().update_all(total_count);
                    if let Some(tag_list_model) = tag_list_model {
                        if tag_list_model.is_empty() {
                            sidebar.read().hide_taglist();
                        } else {
                            sidebar.write().update_taglist(tag_list_model);
                            sidebar.read().show_taglist();
                        }
                    } else {
                        sidebar.read().hide_taglist();
                    }
                }
            }
        });

        thread_pool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    pub fn article_view_scroll_diff(&self, diff: f64) -> Result<(), ContentPageError> {
        self.article_view
            .animate_scroll_diff(diff)
            .context(ContentPageErrorKind::ArticleView)?;
        Ok(())
    }

    pub fn sidebar_select_next_item(&self) -> Result<(), ContentPageError> {
        self.sidebar
            .read()
            .select_next_item()
            .context(ContentPageErrorKind::SidebarSelection)?;
        Ok(())
    }

    pub fn sidebar_select_prev_item(&self) -> Result<(), ContentPageError> {
        self.sidebar
            .read()
            .select_prev_item()
            .context(ContentPageErrorKind::SidebarSelection)?;
        Ok(())
    }
}
