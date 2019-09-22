use failure::{Backtrace, Context, Error, Fail};
use std::fmt;

#[derive(Debug)]
pub struct SettingsError {
    inner: Context<SettingsErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum SettingsErrorKind {
    #[fail(display = "Failed to open and read config file on disk.")]
    ReadFromDisk,
    #[fail(display = "Failed to write config file to disk.")]
    WriteToDisk,
    #[fail(display = "Content of settings file is not valid.")]
    InvalidJsonContent,
    #[fail(display = "Failed to serialize settings struct.")]
    Serialize,
    #[fail(display = "Failed to create the directory for settings file.")]
    CreateDirectory,
    #[fail(display = "Keybind name not valid.")]
    InvalidKeybind,
    #[fail(display = "Unknown Error")]
    Unknown,
}

impl Fail for SettingsError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl SettingsError {
    #[allow(dead_code)]
    pub fn kind(&self) -> SettingsErrorKind {
        *self.inner.get_context()
    }
}

impl From<SettingsErrorKind> for SettingsError {
    fn from(kind: SettingsErrorKind) -> SettingsError {
        SettingsError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<SettingsErrorKind>> for SettingsError {
    fn from(inner: Context<SettingsErrorKind>) -> SettingsError {
        SettingsError { inner }
    }
}

impl From<Error> for SettingsError {
    fn from(_: Error) -> SettingsError {
        SettingsError {
            inner: Context::new(SettingsErrorKind::Unknown),
        }
    }
}
