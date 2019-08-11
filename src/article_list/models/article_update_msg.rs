use news_flash::models::{ArticleID, Marked, Read};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadUpdate {
    pub article_id: ArticleID,
    pub read: Read,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkUpdate {
    pub article_id: ArticleID,
    pub marked: Marked,
}
