use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct ColorError {
    inner: Context<ColorErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ColorErrorKind {
    #[fail(display = "Color string containing illegal character")]
    IllegalCharacter,
    #[fail(display = "Error converting rgb color to hsla")]
    RgbToHsla,
    #[fail(display = "Error converting hsla color to rgb")]
    HslaToRgb,
    #[fail(display = "Error parsing color string")]
    Parse,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for ColorError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl ColorError {
    #[allow(dead_code)]
    pub fn kind(&self) -> ColorErrorKind {
        *self.inner.get_context()
    }
}

impl From<ColorErrorKind> for ColorError {
    fn from(kind: ColorErrorKind) -> ColorError {
        ColorError { inner: Context::new(kind) }
    }
}

impl From<Context<ColorErrorKind>> for ColorError {
    fn from(inner: Context<ColorErrorKind>) -> ColorError {
        ColorError { inner }
    }
}

impl From<Error> for ColorError {
    fn from(_: Error) -> ColorError {
        ColorError {
            inner: Context::new(ColorErrorKind::Unknown),
        }
    }
}
