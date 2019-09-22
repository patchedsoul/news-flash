use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct TagListModelError {
    inner: Context<TagListModelErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum TagListModelErrorKind {
    #[fail(display = "Tag is already present in list")]
    AlreadyExists,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for TagListModelError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for TagListModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl TagListModelError {
    #[allow(dead_code)]
    pub fn kind(&self) -> TagListModelErrorKind {
        *self.inner.get_context()
    }
}

impl From<TagListModelErrorKind> for TagListModelError {
    fn from(kind: TagListModelErrorKind) -> TagListModelError {
        TagListModelError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<TagListModelErrorKind>> for TagListModelError {
    fn from(inner: Context<TagListModelErrorKind>) -> TagListModelError {
        TagListModelError { inner }
    }
}

impl From<Error> for TagListModelError {
    fn from(_: Error) -> TagListModelError {
        TagListModelError {
            inner: Context::new(TagListModelErrorKind::Unknown),
        }
    }
}
