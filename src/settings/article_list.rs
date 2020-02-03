use news_flash::models::ArticleOrder;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleListSettings {
    pub order: ArticleOrder,
}

impl Default for ArticleListSettings {
    fn default() -> Self {
        ArticleListSettings {
            order: ArticleOrder::NewestFirst,
        }
    }
}
