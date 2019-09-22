use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct TagListError {
    inner: Context<TagListErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum TagListErrorKind {
    #[fail(display = "Can't select invalid/non existant tag")]
    InvalidSelection,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for TagListError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for TagListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl TagListError {
    #[allow(dead_code)]
    pub fn kind(&self) -> TagListErrorKind {
        *self.inner.get_context()
    }
}

impl From<TagListErrorKind> for TagListError {
    fn from(kind: TagListErrorKind) -> TagListError {
        TagListError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<TagListErrorKind>> for TagListError {
    fn from(inner: Context<TagListErrorKind>) -> TagListError {
        TagListError { inner }
    }
}

impl From<Error> for TagListError {
    fn from(_: Error) -> TagListError {
        TagListError {
            inner: Context::new(TagListErrorKind::Unknown),
        }
    }
}
