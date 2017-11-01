use std::error;
use std::fmt;
use std::io;

use scan::ScanError;

#[derive(Debug)]
pub enum Error {
    /// I/O Error
    Io(io::Error),
    UnrecognizedToken,
    UnterminatedLiteral,
    UnterminatedBracket,
    UnterminatedBlockComment,
    BadVariableName,
    BadNumber,
    ExpectedEqualsSign,
    MalformedBlobLiteral,
    MalformedHexInteger,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::UnrecognizedToken => write!(f, "unrecognized token"),
            Error::UnterminatedLiteral => write!(f, "non-terminated literal"),
            Error::UnterminatedBracket => write!(f, "non-terminated bracket"),
            Error::UnterminatedBlockComment => write!(f, "non-terminated block comment"),
            Error::BadVariableName => write!(f, "bad variable name"),
            Error::BadNumber => write!(f, "bad number"),
            Error::ExpectedEqualsSign => write!(f, "expected = sign"),
            Error::MalformedBlobLiteral => write!(f, "malformed blob literal"),
            Error::MalformedHexInteger => write!(f, "malformed hex integer"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::UnrecognizedToken => "Unrecognized token",
            Error::UnterminatedLiteral => "Unterminated literal",
            Error::UnterminatedBracket => "Unterminated bracket",
            Error::UnterminatedBlockComment => "Unterminated block comment",
            Error::BadVariableName => "Bad variable name",
            Error::BadNumber => "Bad number",
            Error::ExpectedEqualsSign => "Expected = sign",
            Error::MalformedBlobLiteral => "Malformed blob literal",
            Error::MalformedHexInteger => "Malformed hex integer",
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl ScanError for Error {}
