use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct LoginScreenError {
    inner: Context<LoginScreenErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum LoginScreenErrorKind {
    #[fail(display = "Error loading plugin icon")]
    Icon,
    #[fail(display = "Wrong LoginGUI description")]
    LoginGUI,
    #[fail(display = "No/Invalid OAuth login url")]
    OauthUrl,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for LoginScreenError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for LoginScreenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl LoginScreenError {
    #[allow(dead_code)]
    pub fn kind(&self) -> LoginScreenErrorKind {
        *self.inner.get_context()
    }
}

impl From<LoginScreenErrorKind> for LoginScreenError {
    fn from(kind: LoginScreenErrorKind) -> LoginScreenError {
        LoginScreenError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<LoginScreenErrorKind>> for LoginScreenError {
    fn from(inner: Context<LoginScreenErrorKind>) -> LoginScreenError {
        LoginScreenError { inner }
    }
}

impl From<Error> for LoginScreenError {
    fn from(_: Error) -> LoginScreenError {
        LoginScreenError {
            inner: Context::new(LoginScreenErrorKind::Unknown),
        }
    }
}
