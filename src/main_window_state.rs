use crate::sidebar::models::SidebarSelection;
use crate::content_page::HeaderSelection;
use news_flash::models::ArticleOrder;

#[derive(Clone, Debug)]
pub struct MainWindowState {
    pub sidebar: SidebarSelection,
    pub header: HeaderSelection,
    pub search_term: Option<String>,
    pub article_list_order: ArticleOrder,
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