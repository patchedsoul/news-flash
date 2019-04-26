use chrono::NaiveDateTime;
use news_flash::models::{Article, ArticleID, FavIcon, FeedID, Marked, Read};

#[derive(Debug, Clone)]
pub struct ArticleListArticleModel {
    pub id: ArticleID,
    pub title: String,
    pub feed_id: FeedID,
    pub feed_title: String,
    pub favicon: Option<FavIcon>,
    pub date: NaiveDateTime,
    pub summary: String,
    pub read: Read,
    pub marked: Marked,
}

impl ArticleListArticleModel {
    pub fn new(article: Article, feed_title: String, favicon: Option<FavIcon>) -> Self {
        let (article_id, title, _author, feed_id, _url, date, summary, _direction, read, marked) = article.decompose();

        ArticleListArticleModel {
            id: article_id,
            title: match title {
                Some(title) => title,
                None => "No Title".to_owned(),
            },
            feed_id,
            feed_title,
            favicon,
            date,
            summary: match summary {
                Some(summary) => summary,
                None => "No Summary".to_owned(),
            },
            read,
            marked,
        }
    }
}

impl PartialEq for ArticleListArticleModel {
    fn eq(&self, other: &ArticleListArticleModel) -> bool {
        self.id == other.id
    }
}
