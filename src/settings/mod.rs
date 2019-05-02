mod article_list;
mod article_view;
mod sidebar;

use serde_derive::{Deserialize, Serialize};
use article_list::ArticleListSettings;
use article_view::ArticleViewSettings;
use sidebar::SidebarSettings;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    article_list: ArticleListSettings,
    article_view: ArticleViewSettings,
    sidebar: SidebarSettings,
}