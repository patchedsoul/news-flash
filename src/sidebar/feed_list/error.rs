use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct FeedListError {
    inner: Context<FeedListErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum FeedListErrorKind {
    #[fail(display = "Failed to load embeded file")]
    EmbedFile,
    #[fail(display = "Failed to find widget in UI file")]
    UIFile,
    #[fail(display = "Category is not present in list")]
    CategoryNotFound,
    #[fail(display = "Feed is not present in list")]
    FeedNotFound,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for FeedListError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for FeedListError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<FeedListErrorKind> for FeedListError {
    fn from(kind: FeedListErrorKind) -> FeedListError {
        FeedListError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<FeedListErrorKind>> for FeedListError {
    fn from(inner: Context<FeedListErrorKind>) -> FeedListError {
        FeedListError { inner }
    }
}

impl From<Error> for FeedListError {
    fn from(_: Error) -> FeedListError {
        FeedListError {
            inner: Context::new(FeedListErrorKind::Unknown),
        }
    }
}
