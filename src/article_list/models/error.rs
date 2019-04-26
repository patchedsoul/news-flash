use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct ArticleListModelError {
    inner: Context<ArticleListModelErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ArticleListModelErrorKind {
    #[fail(display = "Listmodel already contains article with identical ID")]
    AlreadyContainsArticle,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for ArticleListModelError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for ArticleListModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl ArticleListModelError {
    #[allow(dead_code)]
    pub fn kind(&self) -> ArticleListModelErrorKind {
        *self.inner.get_context()
    }
}

impl From<ArticleListModelErrorKind> for ArticleListModelError {
    fn from(kind: ArticleListModelErrorKind) -> ArticleListModelError {
        ArticleListModelError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ArticleListModelErrorKind>> for ArticleListModelError {
    fn from(inner: Context<ArticleListModelErrorKind>) -> ArticleListModelError {
        ArticleListModelError { inner }
    }
}

impl From<Error> for ArticleListModelError {
    fn from(_: Error) -> ArticleListModelError {
        ArticleListModelError {
            inner: Context::new(ArticleListModelErrorKind::Unknown),
        }
    }
}
