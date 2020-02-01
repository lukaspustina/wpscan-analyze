use failure::{Backtrace, Context, Fail};
use std::fmt;

/// The error kind for errors that get returned in the crate
#[derive(Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "data has wrong format")]
    InvalidFormat,
    #[fail(display = "invalid file '{}'", _0)]
    InvalidFile(String),
    #[fail(display = "invalid output format '{}'", _0)]
    InvalidOutputFormat(String),
    #[fail(display = "invalid output detail type '{}'", _0)]
    InvalidOutputDetail(String),
    #[fail(display = "output failed")]
    OutputFailed,
    #[fail(display = "WpScan is not sane, because {}", _0)]
    InsaneWpScan(String),
}

impl Clone for ErrorKind {
    fn clone(&self) -> Self {
        use self::ErrorKind::*;
        match *self {
            InvalidFormat => InvalidFormat,
            InvalidFile(ref s) => InvalidFile(s.clone()),
            InvalidOutputFormat(ref s) => InvalidOutputFormat(s.clone()),
            InvalidOutputDetail(ref s) => InvalidOutputDetail(s.clone()),
            OutputFailed => OutputFailed,
            InsaneWpScan(ref s) => InsaneWpScan(s.clone()),
        }
    }
}

/// The error type for errors that get returned in the lookup module
#[derive(Debug)]
pub struct Error {
    pub(crate) inner: Context<ErrorKind>,
}

impl Error {
    /// Get the kind of the error
    pub fn kind(&self) -> &ErrorKind { self.inner.get_context() }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Error {
            inner: Context::new(self.inner.get_context().clone()),
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> { self.inner.cause() }

    fn backtrace(&self) -> Option<&Backtrace> { self.inner.backtrace() }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt::Display::fmt(&self.inner, f) }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error { Error { inner } }
}

pub type Result<T> = ::std::result::Result<T, Error>;
