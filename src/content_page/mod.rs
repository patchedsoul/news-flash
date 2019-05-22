mod content_header;
mod header_selection;

pub use self::content_header::ContentHeader;
pub use self::header_selection::HeaderSelection;

use crate::article_list::{ArticleList, ArticleListModel, ArticleListArticleModel};
use crate::article_view::ArticleView;
use crate::main_window_state::MainWindowState;
use crate::sidebar::models::SidebarSelection;
use crate::sidebar::{FeedListTree, SideBar, TagListModel};
use crate::util::{BuilderHelper, GtkHandle, GtkUtil};
use crate::settings::Settings;
use failure::format_err;
use failure::Error;
use gio::{ActionExt, ActionMapExt};
use glib::Variant;
use gtk::{Box, BoxExt, Paned, PanedExt};
use news_flash::models::{Article, ArticleID, Marked, PluginID, PluginCapabilities, Read};
use news_flash::NewsFlash;

const SIDEBAR_PANED_DEFAULT_POS: i32 = 220;

pub struct ContentPage {
    page: Box,
    paned: Paned,
    sidebar: SideBar,
    article_list: ArticleList,
    article_view: ArticleView,
    settings: GtkHandle<Settings>,
}

impl ContentPage {
    pub fn new(settings: &GtkHandle<Settings>) -> Result<Self, Error> {
        let builder = BuilderHelper::new("content_page");
        let page = builder.get::<Box>("page");
        let feed_list_box = builder.get::<Box>("feedlist_box");
        let article_list_box = builder.get::<Box>("articlelist_box");
        let articleview_box = builder.get::<Box>("articleview_box");
        let paned = builder.get::<Paned>("paned_lists_article_view");
        let sidebar_paned = builder.get::<Paned>("paned_lists");
        sidebar_paned.set_position(SIDEBAR_PANED_DEFAULT_POS);

        paned.connect_property_position_notify(|paned| {
            if let Ok(main_window) = GtkUtil::get_main_window(paned) {
                if let Some(action) = main_window.lookup_action("sync-paned") {
                    let pos = Variant::from(&paned.get_position());
                    action.activate(Some(&pos));
                }
            }
        });

        let sidebar = SideBar::new()?;
        let article_list = ArticleList::new(settings)?;
        let article_view = ArticleView::new(settings)?;

        feed_list_box.pack_start(&sidebar.widget(), false, true, 0);
        article_list_box.pack_start(&article_list.widget(), false, true, 0);
        articleview_box.pack_start(&article_view.widget(), false, true, 0);

        let settings = settings.clone();

        Ok(ContentPage {
            page,
            paned,
            sidebar,
            article_list,
            article_view,
            settings,
        })
    }

    pub fn widget(&self) -> gtk::Box {
        self.page.clone()
    }

    pub fn set_service(&self, id: &PluginID, user_name: Option<String>) -> Result<(), Error> {
        self.sidebar.set_service(id, user_name)?;
        Ok(())
    }

    pub fn set_paned(&self, pos: i32) {
        self.paned.set_position(pos);
    }

    pub fn update_article_list(
        &mut self,
        news_flash_handle: &GtkHandle<Option<NewsFlash>>,
        window_state: &GtkHandle<MainWindowState>,
    ) {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            self.update_article_list_from_ref(news_flash, &mut *window_state.borrow_mut());
        }
    }

    fn update_article_list_from_ref(
        &mut self,
        news_flash: &mut NewsFlash,
        window_state: &mut MainWindowState,
    ) {
        let relevant_articles_loaded = self.article_list.get_relevant_article_count(window_state.get_header_selection());
        let limit = if window_state.reset_article_list() {
            MainWindowState::page_size()
        } else if relevant_articles_loaded as i64 >= MainWindowState::page_size() {
            relevant_articles_loaded as i64
        } else {
            MainWindowState::page_size()
        };
        let mut list_model = ArticleListModel::new(&self.settings.borrow().get_article_list_order());
        let mut articles = Self::load_articles(news_flash, &window_state, &self.settings, limit, None);

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
    ) -> Result<(), Error> {
        let window_state = window_state.borrow().clone();
        let relevant_articles_loaded = self.article_list.get_relevant_article_count(window_state.get_header_selection());
        let mut list_model = ArticleListModel::new(&self.settings.borrow().get_article_list_order());
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            let mut articles = Self::load_articles(
                news_flash,
                &window_state,
                &self.settings,
                MainWindowState::page_size(),
                Some(relevant_articles_loaded as i64)
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
        limit: i64,
        offset: Option<i64>
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
            SidebarSelection::Feed(id) => Some(id.clone()),
        };
        let category = match &window_state.get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_) | SidebarSelection::Tag(_) => None,
            SidebarSelection::Cateogry(id) => Some(id.clone()),
        };
        let tag = match &window_state.get_sidebar_selection() {
            SidebarSelection::All | SidebarSelection::Feed(_) | SidebarSelection::Cateogry(_) => None,
            SidebarSelection::Tag(id) => Some(id.clone()),
        };

        news_flash
            .get_articles(
                Some(limit),
                offset,
                Some(settings.borrow().get_article_list_order()),
                unread,
                marked,
                feed,
                category,
                tag,
                None,
                None,
                None,
            )
            .unwrap()
    }

    pub fn update_sidebar(&mut self, news_flash_handle: &GtkHandle<Option<NewsFlash>>, state: &GtkHandle<MainWindowState>) {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            self.update_sidebar_from_ref(news_flash, &*state.borrow());
        }
    }

    fn update_sidebar_from_ref(&mut self, news_flash: &mut NewsFlash, state: &MainWindowState) {
        // feedlist
        let mut tree = FeedListTree::new();
        let categories = news_flash.get_categories().unwrap();
        for category in categories {
            let count = match state.get_header_selection() {
                HeaderSelection::Marked => news_flash.marked_count_category(&category.category_id).unwrap(),
                HeaderSelection::All |
                HeaderSelection::Unread => news_flash.unread_count_category(&category.category_id).unwrap(),
            };
            tree.add_category(&category, count as i32).unwrap();
        }
        let (feeds, mappings) = news_flash.get_feeds().unwrap();
        for mapping in mappings {
            let count = match state.get_header_selection() {
                HeaderSelection::Marked => news_flash.marked_count_feed(&mapping.feed_id).unwrap(),
                HeaderSelection::All |
                HeaderSelection::Unread => news_flash.unread_count_feed(&mapping.feed_id).unwrap(),
            };
            let feed = feeds.iter().find(|feed| feed.feed_id == mapping.feed_id).unwrap();
            let favicon = match news_flash.get_icon_info(&feed) {
                Ok(favicon) => Some(favicon),
                Err(_) => None,
            };
            tree.add_feed(&feed, &mapping, count as i32, favicon).unwrap();
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
                    let count = match state.get_header_selection() {
                        HeaderSelection::All |
                        HeaderSelection::Unread => news_flash.unread_count_tag(&tag.tag_id).unwrap(),
                        HeaderSelection::Marked => news_flash.marked_count_tag(&tag.tag_id).unwrap(),
                    };
                    list.add(&tag, count as i32).unwrap();
                }

                self.sidebar.update_taglist(list);
                self.sidebar.show_taglist();
            }
        }

        let total_count = match state.get_header_selection() {
            HeaderSelection::All |
            HeaderSelection::Unread => news_flash.unread_count_all().unwrap(),
            HeaderSelection::Marked => news_flash.marked_count_all().unwrap(),
        };

        self.sidebar.update_feedlist(tree);
        self.sidebar.update_unread_all(total_count);
    }

    pub fn show_article(
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

    pub fn redraw_article(&mut self) -> Result<(), Error> {
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
}
