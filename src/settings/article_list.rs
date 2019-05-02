use serde_derive::{Deserialize, Serialize};
use news_flash::models::ArticleOrder;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleListSettings {
    order: ArticleOrder,
}