use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct FeedListModelError {
    inner: Context<FeedListModelErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum FeedListModelErrorKind {
    #[fail(display = "DnD")]
    DnD,
    #[fail(display = "Category already exists in tree")]
    AddDuplicateCategory,
    #[fail(display = "Parent of category not found in tree")]
    AddCategoryNoParent,
    #[fail(display = "Feed already exists in tree")]
    AddDuplicateFeed,
    #[fail(display = "Parent of feed not found in tree")]
    AddFeedNoParent,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for FeedListModelError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for FeedListModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<FeedListModelErrorKind> for FeedListModelError {
    fn from(kind: FeedListModelErrorKind) -> FeedListModelError {
        FeedListModelError { inner: Context::new(kind) }
    }
}

impl From<Context<FeedListModelErrorKind>> for FeedListModelError {
    fn from(inner: Context<FeedListModelErrorKind>) -> FeedListModelError {
        FeedListModelError { inner: inner }
    }
}

impl From<Error> for FeedListModelError {
    fn from(_: Error) -> FeedListModelError {
        FeedListModelError {
            inner: Context::new(FeedListModelErrorKind::Unknown),
        }
    }
}
