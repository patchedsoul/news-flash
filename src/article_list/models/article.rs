use chrono::NaiveDateTime;
use news_flash::models::{Article, ArticleID, Feed, FeedID, Marked, Read, Url};

#[derive(Debug, Clone)]
pub struct ArticleListArticleModel {
    pub id: ArticleID,
    pub title: String,
    pub feed_id: FeedID,
    pub feed_title: String,
    pub date: NaiveDateTime,
    pub summary: String,
    pub read: Read,
    pub marked: Marked,
    pub url: Option<Url>,
    pub news_flash_feed: Feed,
}

impl ArticleListArticleModel {
    pub fn new(article: Article, feed: &Feed) -> Self {
        let (article_id, title, _author, feed_id, url, date, summary, _direction, read, marked) = article.decompose();

        ArticleListArticleModel {
            id: article_id,
            title: match title {
                Some(title) => title,
                None => "No Title".to_owned(),
            },
            feed_id,
            feed_title: feed.label.clone(),
            date,
            summary: match summary {
                Some(summary) => summary,
                None => "No Summary".to_owned(),
            },
            read,
            marked,
            url,
            news_flash_feed: feed.clone(),
        }
    }
}

impl PartialEq for ArticleListArticleModel {
    fn eq(&self, other: &ArticleListArticleModel) -> bool {
        self.id == other.id
    }
}
