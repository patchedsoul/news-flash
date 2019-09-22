use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct ArticleListError {
    inner: Context<ArticleListErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ArticleListErrorKind {
    #[fail(display = "Article List model error")]
    Model,
    #[fail(display = "Article List is in empty state")]
    EmptyState,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for ArticleListError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for ArticleListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl ArticleListError {
    #[allow(dead_code)]
    pub fn kind(&self) -> ArticleListErrorKind {
        *self.inner.get_context()
    }
}

impl From<ArticleListErrorKind> for ArticleListError {
    fn from(kind: ArticleListErrorKind) -> ArticleListError {
        ArticleListError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ArticleListErrorKind>> for ArticleListError {
    fn from(inner: Context<ArticleListErrorKind>) -> ArticleListError {
        ArticleListError { inner }
    }
}

impl From<Error> for ArticleListError {
    fn from(_: Error) -> ArticleListError {
        ArticleListError {
            inner: Context::new(ArticleListErrorKind::Unknown),
        }
    }
}
