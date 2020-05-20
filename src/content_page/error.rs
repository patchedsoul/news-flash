use failure::{Backtrace, Context, Error, Fail};
use news_flash::models::FeedID;
use std::fmt;

#[derive(Debug)]
pub struct ContentPageError {
    inner: Context<ContentPageErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ContentPageErrorKind {
    #[fail(display = "Error computing article view stuff")]
    ArticleView,
    #[fail(display = "Error computing article list model")]
    ArticleList,
    #[fail(display = "Failed to load data from the database")]
    DataBase,
    #[fail(display = "Failed to find feed for id")]
    MissingFeed(FeedID),
    #[fail(display = "Failed to set sidebar service icon")]
    SidebarService,
    #[fail(display = "Error computing sidebar models")]
    SidebarModels,
    #[fail(display = "Error computing sidebar selection")]
    SidebarSelection,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for ContentPageError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for ContentPageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl ContentPageError {
    #[allow(dead_code)]
    pub fn kind(&self) -> ContentPageErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<ContentPageErrorKind> for ContentPageError {
    fn from(kind: ContentPageErrorKind) -> ContentPageError {
        ContentPageError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ContentPageErrorKind>> for ContentPageError {
    fn from(inner: Context<ContentPageErrorKind>) -> ContentPageError {
        ContentPageError { inner }
    }
}

impl From<Error> for ContentPageError {
    fn from(_: Error) -> ContentPageError {
        ContentPageError {
            inner: Context::new(ContentPageErrorKind::Unknown),
        }
    }
}
