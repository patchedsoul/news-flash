use news_flash::models::ArticleOrder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleListSettings {
    pub order: ArticleOrder,
}

impl ArticleListSettings {
    pub fn default() -> Self {
        ArticleListSettings {
            order: ArticleOrder::NewestFirst,
        }
    }
}
