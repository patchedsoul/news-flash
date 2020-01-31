use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct SidebarError {
    inner: Context<SidebarErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum SidebarErrorKind {
    #[fail(display = "Failed to calculate sidebar selection")]
    Selection,
    #[fail(display = "Error in Plugin meta data")]
    MetaData,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for SidebarError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for SidebarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl SidebarError {
    #[allow(dead_code)]
    pub fn kind(&self) -> SidebarErrorKind {
        (*self.inner.get_context()).clone()
    }
}

impl From<SidebarErrorKind> for SidebarError {
    fn from(kind: SidebarErrorKind) -> SidebarError {
        SidebarError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<SidebarErrorKind>> for SidebarError {
    fn from(inner: Context<SidebarErrorKind>) -> SidebarError {
        SidebarError { inner }
    }
}

impl From<Error> for SidebarError {
    fn from(_: Error) -> SidebarError {
        SidebarError {
            inner: Context::new(SidebarErrorKind::Unknown),
        }
    }
}
