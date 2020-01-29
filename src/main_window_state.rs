use crate::content_page::HeaderSelection;
use crate::sidebar::models::SidebarSelection;

#[derive(Clone, Debug)]
pub struct MainWindowState {
    sidebar: SidebarSelection,
    header: HeaderSelection,
    search_term: Option<String>,
    reset_article_list: bool,
    offline: bool,
}

const ARTICLE_LIST_PAGE_SIZE: i64 = 20;

impl MainWindowState {
    pub fn new() -> Self {
        MainWindowState {
            sidebar: SidebarSelection::All,
            header: HeaderSelection::All,
            search_term: None,
            reset_article_list: false,
            offline: false,
        }
    }

    pub fn page_size() -> i64 {
        ARTICLE_LIST_PAGE_SIZE
    }

    pub fn reset_article_list(&mut self) -> bool {
        let reset_article_list = self.reset_article_list;
        self.reset_article_list = false;
        reset_article_list
    }

    pub fn get_sidebar_selection(&self) -> &SidebarSelection {
        &self.sidebar
    }

    pub fn set_sidebar_selection(&mut self, sidebar: SidebarSelection) {
        self.sidebar = sidebar;
        self.reset_article_list = true;
    }

    pub fn get_header_selection(&self) -> &HeaderSelection {
        &self.header
    }

    pub fn set_header_selection(&mut self, header: HeaderSelection) {
        self.header = header;
        self.reset_article_list = true;
    }

    #[allow(dead_code)]
    pub fn get_search_term(&self) -> &Option<String> {
        &self.search_term
    }

    pub fn set_search_term(&mut self, search_term: Option<String>) {
        self.search_term = search_term;
        self.reset_article_list = true;
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    pub fn get_offline(&self) -> bool {
        self.offline
    }
}

impl PartialEq for MainWindowState {
    fn eq(&self, other: &MainWindowState) -> bool {
        if self.sidebar != other.sidebar {
            return false;
        }
        if self.header != other.header {
            return false;
        }
        match &self.search_term {
            Some(self_search_term) => match &other.search_term {
                Some(other_search_term) => {
                    if self_search_term != other_search_term {
                        return false;
                    }
                }
                None => return false,
            },
            None => match &other.search_term {
                Some(_) => return false,
                None => {}
            },
        }
        true
    }
}
