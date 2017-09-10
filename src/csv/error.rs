use std::error;
use std::fmt;
use std::io;

use scan::ScanError;

/// Enum listing possible errors from Scanner.
#[derive(Debug)]
pub enum Error {
    /// I/O Error
    Io(io::Error),
    UnescapedQuote(u8),
    UnterminatedQuotedField,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::UnescapedQuote(quote) => write!(f, "unescaped '{}' character", quote as char),
            Error::UnterminatedQuotedField => write!(f, "non-terminated quoted field"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::UnescapedQuote(_) => "Unescaped quote",
            Error::UnterminatedQuotedField => "Unterminated quoted field",
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl ScanError for Error {}
