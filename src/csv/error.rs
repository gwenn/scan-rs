use std::error;
use std::fmt;
use std::io;

use scan::ScanError;

/// Enum listing possible errors from Scanner.
#[derive(Debug)]
pub enum Error {
    /// I/O Error
    Io(io::Error),
    UnescapedQuote {
        quote: u8,
        pos: Option<(u64, usize)>,
    },
    UnterminatedQuotedField(Option<(u64, usize)>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::UnescapedQuote { quote, pos } => write!(
                f,
                "unescaped '{}' character at {:?}",
                quote as char,
                pos.unwrap()
            ),
            Error::UnterminatedQuotedField(pos) => {
                write!(f, "non-terminated quoted field at {:?}", pos.unwrap())
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::UnescapedQuote { .. } => "Unescaped quote",
            Error::UnterminatedQuotedField(_) => "Unterminated quoted field",
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl ScanError for Error {
    fn position(&mut self, line: u64, column: usize) {
        match *self {
            Error::UnescapedQuote { ref mut pos, .. } => *pos = Some((line, column)),
            Error::UnterminatedQuotedField(ref mut pos) => *pos = Some((line, column)),
            _ => {}
        }
    }
}
