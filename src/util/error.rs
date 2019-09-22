use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct UtilError {
    inner: Context<UtilErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum UtilErrorKind {
    #[fail(display = "Error creating/opening file")]
    CreateFile,
    #[fail(display = "Error writing file to disc")]
    WriteFile,
    #[fail(display = "Provided widget is not a main window")]
    WidgetIsMainwindow,
    #[fail(display = "Failed to create a cairo surface from the given data")]
    CairoSurface,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for UtilError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for UtilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl UtilError {
    #[allow(dead_code)]
    pub fn kind(&self) -> UtilErrorKind {
        *self.inner.get_context()
    }
}

impl From<UtilErrorKind> for UtilError {
    fn from(kind: UtilErrorKind) -> UtilError {
        UtilError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<UtilErrorKind>> for UtilError {
    fn from(inner: Context<UtilErrorKind>) -> UtilError {
        UtilError { inner }
    }
}

impl From<Error> for UtilError {
    fn from(_: Error) -> UtilError {
        UtilError {
            inner: Context::new(UtilErrorKind::Unknown),
        }
    }
}
