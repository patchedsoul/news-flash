use crate::sidebar::models::SidebarSelection;
use crate::content_page::HeaderSelection;
use news_flash::models::ArticleOrder;

#[derive(Clone, Debug)]
pub struct MainWindowState {
    sidebar: SidebarSelection,
    header: HeaderSelection,
    search_term: Option<String>,
    article_list_order: ArticleOrder,
    articles_showing: i64,
}

const ARTICLE_LIST_PAGE_SIZE: i64 = 20;

impl MainWindowState {
    pub fn new() -> Self {
        MainWindowState {
            sidebar: SidebarSelection::All,
            header: HeaderSelection::All,
            search_term: None,
            article_list_order: ArticleOrder::NewestFirst,
            articles_showing: ARTICLE_LIST_PAGE_SIZE,
        }
    }

    pub fn show_more(&mut self) {
        self.articles_showing += ARTICLE_LIST_PAGE_SIZE;
    }

    fn reset_article_list_size(&mut self) {
        self.articles_showing = ARTICLE_LIST_PAGE_SIZE / 2;
    }

    pub fn get_sidebar_selection(&self) -> &SidebarSelection {
        &self.sidebar
    }

    pub fn set_sidebar_selection(&mut self, sidebar: SidebarSelection) {
        self.sidebar = sidebar;
        self.reset_article_list_size();
    }

    pub fn get_header_selection(&self) -> &HeaderSelection {
        &self.header
    }

    pub fn set_header_selection(&mut self, header: HeaderSelection) {
        self.header = header;
        self.reset_article_list_size();
    }

    pub fn get_search_term(&self) -> &Option<String> {
        &self.search_term
    }

    pub fn set_search_term(&mut self, search_term: Option<String>) {
        self.search_term = search_term;
        self.reset_article_list_size();
    }

    pub fn get_article_list_order(&self) -> &ArticleOrder {
        &self.article_list_order
    }

    pub fn set_article_list_order(&mut self, order: ArticleOrder) {
        self.article_list_order = order;
        self.reset_article_list_size();
    }

    pub fn get_articles_showing(&self) -> i64 {
        self.articles_showing
    }
}

impl PartialEq for MainWindowState {
    fn eq(&self, other: &MainWindowState) -> bool {
        if self.sidebar != other.sidebar {
            return false
        }
        if self.header != other.header {
            return false
        }
        match &self.search_term {
            Some(self_search_term) => {
                match &other.search_term {
                    Some(other_search_term) => {
                        if self_search_term != other_search_term {
                            return false
                        }
                    },
                    None => return false,
                }
            },
            None => {
                match &other.search_term {
                    Some(_) => return false,
                    None => {},
                }
            },
        }
        if self.article_list_order != other.article_list_order {
            return false
        }
        true
    }
}