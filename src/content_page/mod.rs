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
use glib::{clone, Sender};
use gtk::{Box, BoxExt, WidgetExt};
use libhandy::Leaflet;
use news_flash::models::{Article, ArticleFilter, Marked, PluginCapabilities, PluginID, Read, NEWSFLASH_TOPLEVEL};
use news_flash::NewsFlash;
use parking_lot::RwLock;
use std::collections::HashSet;
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
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
    ) -> Self {
        let feed_list_box = builder.get::<Box>("feedlist_box");
        let article_list_box = builder.get::<Box>("articlelist_box");
        let articleview_box = builder.get::<Box>("articleview_box");

        // workaround
        let minor_leaflet = builder.get::<Leaflet>("minor_leaflet");
        minor_leaflet.set_hexpand(false);

        let sidebar = Arc::new(RwLock::new(SideBar::new(state, sender.clone(), features)));
        let article_list = Arc::new(RwLock::new(ArticleList::new(
            settings,
            content_header,
            state,
            sender.clone(),
        )));
        let article_view = ArticleView::new(settings, &sender);

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
        self.sidebar
            .write()
            .update_feedlist(feed_tree_model, &Arc::new(RwLock::new(None)));

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
        threadpool: ThreadPool,
    ) {
        let (sender, receiver) = oneshot::channel::<Result<ArticleListModel, ContentPageErrorKind>>();

        let relevant_articles_loaded = self
            .article_list
            .read()
            .get_relevant_article_count(self.state.read().get_header_selection());

        let news_flash = news_flash.clone();
        let window_state = self.state.clone();
        let current_undo_action = undo_bar.get_current_action();
        let processing_undo_actions = undo_bar.processing_actions();
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
                    &processing_undo_actions,
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

        let glib_future = receiver.map(clone!(
            @weak self.state as window_state,
            @weak self.article_list as article_list => @default-panic, move |res|
        {
            if let Ok(res) = res {
                if let Ok(article_list_model) = res {
                    article_list.write().update(article_list_model, &window_state);
                }
            }
        }));

        threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    pub fn load_more_articles(
        &self,
        news_flash_handle: &Arc<RwLock<Option<NewsFlash>>>,
        window_state: &Arc<RwLock<MainWindowState>>,
        undo_bar: &UndoBar,
        threadpool: ThreadPool,
    ) {
        let (sender, receiver) = oneshot::channel::<Result<ArticleListModel, ContentPageErrorKind>>();

        let relevant_articles_loaded = self
            .article_list
            .read()
            .get_relevant_article_count(window_state.read().get_header_selection());

        let current_undo_action = undo_bar.get_current_action();
        let processing_undo_actions = undo_bar.processing_actions();
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
                    &processing_undo_actions,
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

        let glib_future = receiver.map(
            clone!(@weak self.article_list as article_list => @default-panic, move |res| {
                if let Ok(res) = res {
                    if let Ok(article_list_model) = res {
                        article_list.write().add_more_articles(article_list_model);
                    }
                }
            }),
        );

        threadpool.spawn_ok(thread_future);
        Util::glib_spawn_future(glib_future);
    }

    fn load_articles(
        news_flash: &NewsFlash,
        window_state: &RwLock<MainWindowState>,
        settings: &Arc<RwLock<Settings>>,
        current_undo_action: &Option<UndoActionModel>,
        processing_undo_actions: &Arc<RwLock<HashSet<UndoActionModel>>>,
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
            SidebarSelection::All | SidebarSelection::Category(_, _) | SidebarSelection::Tag(_, _) => None,
            SidebarSelection::Feed(id, _parent_id, _title) => Some(id.clone()),
        };
        let category = match &window_state.read().get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_, _, _) | SidebarSelection::Tag(_, _) => None,
            SidebarSelection::Category(id, _title) => Some(id.clone()),
        };
        let tag = match &window_state.read().get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_, _, _) | SidebarSelection::Category(_, _) => None,
            SidebarSelection::Tag(id, _title) => Some(id.clone()),
        };
        let search_term = window_state.read().get_search_term().clone();
        let (feed_blacklist, category_blacklist) = {
            let mut undo_actions = Vec::new();
            let mut feed_blacklist = Vec::new();
            let mut category_blacklist = Vec::new();
            if let Some(current_undo_action) = current_undo_action {
                undo_actions.push(current_undo_action);
            }
            let processing_undo_actions = &*processing_undo_actions.read();
            for processing_undo_action in processing_undo_actions {
                undo_actions.push(&processing_undo_action);
            }
            for undo_action in undo_actions {
                match undo_action {
                    UndoActionModel::DeleteFeed(feed_id, _label) => feed_blacklist.push(feed_id.clone()),
                    UndoActionModel::DeleteCategory(category_id, _label) => {
                        category_blacklist.push(category_id.clone())
                    }
                    UndoActionModel::DeleteTag(_tag_id, _label) => {}
                }
            }

            let feed_blacklist = if feed_blacklist.is_empty() {
                None
            } else {
                Some(feed_blacklist)
            };
            let category_blacklist = if category_blacklist.is_empty() {
                None
            } else {
                Some(category_blacklist)
            };

            (feed_blacklist, category_blacklist)
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
        threadpool: ThreadPool,
        features: &Arc<RwLock<Option<PluginCapabilities>>>,
    ) {
        let (sender, receiver) =
            oneshot::channel::<Result<(i64, FeedListTree, Option<TagListModel>), ContentPageErrorKind>>();

        let news_flash = news_flash.clone();
        let state = self.state.clone();
        let current_undo_action = undo_bar.get_current_action();
        let processing_undo_actions = undo_bar.processing_actions();
        let app_features = features.clone();
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

                let mut pending_delte_feeds = HashSet::new();
                let mut pending_delete_categories = HashSet::new();
                let mut pending_delete_tags = HashSet::new();
                if let Some(current_undo_action) = &current_undo_action {
                    match current_undo_action {
                        UndoActionModel::DeleteFeed(id, _label) => pending_delte_feeds.insert(id),
                        UndoActionModel::DeleteCategory(id, _label) => pending_delete_categories.insert(id),
                        UndoActionModel::DeleteTag(id, _label) => pending_delete_tags.insert(id),
                    };
                }
                let processing_undo_actions = &*processing_undo_actions.read();
                for processing_undo_action in processing_undo_actions {
                    match processing_undo_action {
                        UndoActionModel::DeleteFeed(id, _label) => pending_delte_feeds.insert(id),
                        UndoActionModel::DeleteCategory(id, _label) => pending_delete_categories.insert(id),
                        UndoActionModel::DeleteTag(id, _label) => pending_delete_tags.insert(id),
                    };
                }

                // feedlist: Categories
                for category in &categories {
                    if pending_delete_categories.contains(&category.category_id) {
                        continue;
                    }

                    let category_item_count = Util::calculate_item_count_for_category(
                        &category.category_id,
                        &categories,
                        &mappings,
                        &feed_count_map,
                        &pending_delte_feeds,
                        &pending_delete_categories,
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
                    if pending_delte_feeds.contains(&mapping.feed_id)
                        || pending_delete_categories.contains(&mapping.category_id)
                    {
                        continue;
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
                let mut support_tags = false;
                if let Some(features) = app_features.read().as_ref() {
                    support_tags = features.contains(PluginCapabilities::SUPPORT_TAGS);
                }

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
                            if pending_delete_tags.contains(&tag.tag_id) {
                                continue;
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
                    &pending_delte_feeds,
                    &pending_delete_categories,
                );

                sender
                    .send(Ok((total_item_count, tree, tag_list_model)))
                    .expect(CHANNEL_ERROR);
            }
        };

        let glib_future = receiver.map(clone!(
            @weak self.sidebar as sidebar,
            @strong features => @default-panic, move |res| {
            if let Ok(res) = res {
                if let Ok((total_count, feed_list_model, tag_list_model)) = res {
                    sidebar.write().update_feedlist(feed_list_model, &features);
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
        }));

        threadpool.spawn_ok(thread_future);
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
