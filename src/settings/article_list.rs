use serde_derive::{Deserialize, Serialize};
use news_flash::models::ArticleOrder;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleListSettings {
    order: ArticleOrder,
}

impl ArticleListSettings {
    pub fn default() -> Self {
        ArticleListSettings {
            order: ArticleOrder::NewestFirst,
        }
    }

    pub fn get_order(&self) -> ArticleOrder {
        self.order.clone()
    }
}