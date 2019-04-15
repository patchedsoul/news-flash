mod content_header;
mod header_selection;

pub use self::content_header::ContentHeader;
pub use self::header_selection::HeaderSelection;

use crate::article_list::{ArticleList, ArticleListModel};
use crate::article_view::ArticleView;
use crate::main_window_state::MainWindowState;
use crate::sidebar::models::SidebarSelection;
use crate::sidebar::{FeedListTree, SideBar, TagListModel};
use crate::util::GtkHandle;
use crate::util::GtkUtil;
use crate::Resources;
use failure::format_err;
use failure::Error;
use gio::{ActionExt, ActionMapExt};
use glib::Variant;
use gtk::{BoxExt, Builder, PanedExt};
use news_flash::models::{Article, ArticleID, Marked, PluginID, Read};
use news_flash::NewsFlash;
use std::str;

const SIDEBAR_PANED_DEFAULT_POS: i32 = 220;

pub struct ContentPage {
    page: gtk::Box,
    paned: gtk::Paned,
    sidebar: SideBar,
    article_list: ArticleList,
    article_view: ArticleView,
}

impl ContentPage {
    pub fn new() -> Result<Self, Error> {
        let ui_data = Resources::get("ui/content_page.ui").ok_or_else(|| format_err!("some err"))?;
        let ui_string = str::from_utf8(ui_data.as_ref())?;
        let builder = Builder::new_from_string(ui_string);
        let page: gtk::Box = builder.get_object("page").ok_or_else(|| format_err!("some err"))?;
        let feed_list_box: gtk::Box = builder.get_object("feedlist_box").ok_or_else(|| format_err!("some err"))?;
        let article_list_box: gtk::Box = builder.get_object("articlelist_box").ok_or_else(|| format_err!("some err"))?;
        let articleview_box: gtk::Box = builder.get_object("articleview_box").ok_or_else(|| format_err!("some err"))?;
        let paned: gtk::Paned = builder.get_object("paned_lists_article_view").ok_or_else(|| format_err!("some err"))?;
        let sidebar_paned: gtk::Paned = builder.get_object("paned_lists").ok_or_else(|| format_err!("some err"))?;
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
        let article_list = ArticleList::new()?;
        let article_view = ArticleView::new()?;

        feed_list_box.pack_start(&sidebar.widget(), false, true, 0);
        article_list_box.pack_start(&article_list.widget(), false, true, 0);
        articleview_box.pack_start(&article_view.widget(), false, true, 0);

        Ok(ContentPage {
            page,
            paned,
            sidebar,
            article_list,
            article_view,
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

    pub fn update_article_list(&mut self, news_flash_handle: &GtkHandle<Option<NewsFlash>>, window_state: &GtkHandle<MainWindowState>) {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            self.update_article_list_from_ref(news_flash, window_state);
        }
    }

    pub fn update_article_list_from_ref(&mut self, news_flash: &mut NewsFlash, window_state: &GtkHandle<MainWindowState>) {
        let window_state = window_state.borrow().clone();
        let mut list_model = ArticleListModel::new(window_state.get_article_list_order());
        let mut articles = Self::load_articles(news_flash, &window_state, None);

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

    pub fn load_more_articles(&mut self, news_flash_handle: &GtkHandle<Option<NewsFlash>>, window_state: &GtkHandle<MainWindowState>, offset: i64) -> Result<(), Error> {
        let window_state = window_state.borrow().clone();
        let mut list_model = ArticleListModel::new(window_state.get_article_list_order());
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            let mut articles = Self::load_articles(news_flash, &window_state, Some(offset));
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

    fn load_articles(news_flash: &mut NewsFlash, window_state: &MainWindowState, offset: Option<i64>) -> Vec<Article> {
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

        let limit = window_state.get_articles_showing();
        let limit = match offset {
            Some(offset) => Some(limit - offset),
            None => Some(limit),
        };

        news_flash
            .get_articles(
                limit,
                offset,
                Some(window_state.get_article_list_order().clone()),
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

    pub fn update_sidebar(&mut self, news_flash_handle: &GtkHandle<Option<NewsFlash>>) {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            self.update_sidebar_from_ref(news_flash);
        }
    }

    pub fn update_sidebar_from_ref(&mut self, news_flash: &mut NewsFlash) {
        // feedlist
        let mut tree = FeedListTree::new();
        let categories = news_flash.get_categories().unwrap();
        for category in categories {
            let count = news_flash.unread_count_category(&category.category_id).unwrap();
            tree.add_category(&category, count as i32).unwrap();
        }
        let (feeds, mappings) = news_flash.get_feeds().unwrap();
        for mapping in mappings {
            let count = news_flash.unread_count_feed(&mapping.feed_id).unwrap();
            let feed = feeds.iter().find(|feed| feed.feed_id == mapping.feed_id).unwrap();
            let favicon = match news_flash.get_icon_info(&feed) {
                Ok(favicon) => Some(favicon),
                Err(_) => None,
            };
            tree.add_feed(&feed, &mapping, count as i32, favicon).unwrap();
        }

        // tag list
        let mut list = TagListModel::new();
        //let tags = news_flash.get_tags().unwrap();
        let tags = crate::main_window::MainWindow::demo_tag_list();
        for tag in tags {
            let count = news_flash.unread_count_tags(&tag.tag_id).unwrap();
            list.add(&tag, count as i32).unwrap();
        }

        let total_unread = news_flash.unread_count_all().unwrap();

        self.sidebar.update_feedlist(tree);
        self.sidebar.update_taglist(list);
        self.sidebar.update_unread_all(total_unread);
    }

    pub fn show_article(&mut self, article_id: &ArticleID, news_flash_handle: &GtkHandle<Option<NewsFlash>>) -> Result<(), Error> {
        if let Some(news_flash) = news_flash_handle.borrow_mut().as_mut() {
            let article = news_flash.get_fat_article(article_id).unwrap();
            let (feeds, _) = news_flash.get_feeds().unwrap();
            let feed = feeds.iter().find(|&f| f.feed_id == article.feed_id).unwrap();
            self.article_view.show_article(article, feed.label.clone())?;
            return Ok(());
        }
        Err(format_err!("some err"))
    }
}
