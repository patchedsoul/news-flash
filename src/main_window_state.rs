use crate::sidebar::models::SidebarSelection;
use crate::content_page::HeaderSelection;

pub struct MainWindowState {
    pub sidebar: SidebarSelection,
    pub header: HeaderSelection,
    pub search_term: Option<String>,
}