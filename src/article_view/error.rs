use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct ArticleViewError {
    inner: Context<ArticleViewErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ArticleViewErrorKind {
    #[fail(display = "Active webview name is no present in UI")]
    InvalidActiveWebView,
    #[fail(display = "Currently no webview as active view")]
    NoActiveWebView,
    #[fail(display = "Executed JS didn't return any value")]
    NoValueFromJS,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for ArticleViewError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for ArticleViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl ArticleViewError {
    #[allow(dead_code)]
    pub fn kind(&self) -> ArticleViewErrorKind {
        *self.inner.get_context()
    }
}

impl From<ArticleViewErrorKind> for ArticleViewError {
    fn from(kind: ArticleViewErrorKind) -> ArticleViewError {
        ArticleViewError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ArticleViewErrorKind>> for ArticleViewError {
    fn from(inner: Context<ArticleViewErrorKind>) -> ArticleViewError {
        ArticleViewError { inner }
    }
}

impl From<Error> for ArticleViewError {
    fn from(_: Error) -> ArticleViewError {
        ArticleViewError {
            inner: Context::new(ArticleViewErrorKind::Unknown),
        }
    }
}
