use news_flash::models::{ArticleID, Read};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkReadUpdate {
    pub article_id: ArticleID,
    pub read: Read,
}
