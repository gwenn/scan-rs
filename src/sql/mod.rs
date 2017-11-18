//! Adaptation/port of [`SQLite` tokenizer](http://www.sqlite.org/src/artifact?ci=trunk&filename=src/tokenize.c)
use std::result::Result;

use memchr::memchr;

mod error;

pub use sql::error::Error;
pub use scan::Splitter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    // Keywords:
    Abort,
    Action,
    Add,
    After,
    All,
    Alter,
    Analyze,
    And,
    // Any,
    As,
    Asc,
    Attach,
    Autoincr,
    Before,
    Begin,
    Between,
    By,
    Cascade,
    Case,
    Cast,
    Check,
    Collate,
    ColumnKw,
    Commit,
    Conflict,
    Constraint,
    Create,
    Cross,
    CurrentDate,
    CurrentTime,
    CurrentTimestamp,
    Database,
    Default,
    Deferrable,
    Deferred,
    Delete,
    Desc,
    Detach,
    Distinct,
    Drop,
    Each,
    Else,
    End,
    Escape,
    Except,
    Exclusive,
    Exists,
    Explain,
    Fail,
    For,
    Foreign,
    From,
    Full,
    // Function,
    Glob,
    Group,
    Having,
    If,
    Ignore,
    Immediate,
    In,
    Index,
    Indexed,
    Initially,
    Inner,
    Insert,
    Instead,
    Intersect,
    Into,
    Is,
    // IsNot,
    IsNull,
    Join,
    Key,
    Left,
    Like,
    Limit,
    Match,
    Natural,
    No,
    Not,
    NotNull,
    Null,
    Of,
    Offset,
    On,
    Or,
    Order,
    Outer,
    Plan,
    Pragma,
    Primary,
    Query,
    Raise,
    Recursive,
    References,
    Regexp,
    Reindex,
    Release,
    Rename,
    Replace,
    Restrict,
    Right,
    Rollback,
    Row,
    Savepoint,
    Select,
    Set,
    Table,
    Temp,
    Then,
    To,
    Transaction,
    Trigger,
    Union,
    Unique,
    Update,
    Using,
    Vacuum,
    Values,
    View,
    Virtual,
    When,
    Where,
    With,
    Without,

    // Identifiers:
    StringLiteral,
    Id,
    Variable,

    // Values:
    Blob,
    Integer,
    Float,

    // Symbols:
    BitAnd,
    BitNot,
    BitOr,
    Comma,
    Concat,
    Dot,
    Equals,
    GreaterThan,
    GreaterEquals,
    LeftParen,
    LeftShift,
    LessEquals,
    LessThan,
    Minus,
    NotEquals,
    Plus,
    Reminder,
    RightParen,
    RightShift,
    Semi,
    Slash,
    Star,
}

pub type Token<'input> = (&'input [u8], TokenType);

pub struct Tokenizer {}

impl Splitter for Tokenizer {
    type Error = Error;
    type TokenType = TokenType;

    fn split<'input>(
        &mut self,
        data: &'input mut [u8],
        eof: bool,
    ) -> Result<(Option<Token<'input>>, usize), Error> {
        if eof && data.is_empty() {
            return Ok((None, 0));
        }
        if data[0].is_ascii_whitespace() {
            // eat as much space as possible
            return Ok((
                None,
                match data.iter().skip(1).position(|&b| !b.is_ascii_whitespace()) {
                    Some(i) => i,
                    _ => data.len(),
                },
            ));
        }
        match data[0] {
            b'-' => if let Some(b) = data.get(1) {
                if *b == b'-' {
                    // eat comment
                    if let Some(i) = memchr(b'\n', data) {
                        return Ok((None, i));
                    } else if eof {
                        return Ok((None, data.len()));
                    } // else ask more data until '\n'
                } else {
                    return Ok((Some((&data[..1], TokenType::Minus)), 1));
                }
            } else if eof {
                return Ok((Some((&data[..1], TokenType::Minus)), 1));
            }, /* else ask more data */
            b'(' => return Ok((Some((&data[..1], TokenType::LeftParen)), 1)),
            b')' => return Ok((Some((&data[..1], TokenType::RightParen)), 1)),
            b';' => return Ok((Some((&data[..1], TokenType::Semi)), 1)),
            b'+' => return Ok((Some((&data[..1], TokenType::Plus)), 1)),
            b'*' => return Ok((Some((&data[..1], TokenType::Star)), 1)),
            b'/' => if let Some(b) = data.get(1) {
                if *b == b'*' {
                    // eat comment
                    if let Some(i) = data.windows(2).position(|w| w == b"*/") {
                        return Ok((None, i + 2));
                    } else if eof {
                        return Err(Error::UnterminatedBlockComment);
                    } // else ask more data until '*/'
                } else {
                    return Ok((Some((&data[..1], TokenType::Slash)), 1));
                }
            } else if eof {
                return Ok((Some((&data[..1], TokenType::Slash)), 1));
            },
            b'%' => return Ok((Some((&data[..1], TokenType::Reminder)), 1)),
            b'=' => if let Some(b) = data.get(1) {
                return Ok(if *b == b'=' {
                    (Some((&data[..2], TokenType::Equals)), 2)
                } else {
                    (Some((&data[..1], TokenType::Equals)), 1)
                });
            } else if eof {
                return Ok((Some((&data[..1], TokenType::Equals)), 1));
            }, /* else ask more data to fuse '==' or not */
            b'<' => if let Some(b) = data.get(1) {
                return Ok(match *b {
                    b'=' => (Some((&data[..2], TokenType::LessEquals)), 2),
                    b'>' => (Some((&data[..2], TokenType::NotEquals)), 2),
                    b'<' => (Some((&data[..2], TokenType::LeftShift)), 2),
                    _ => (Some((&data[..1], TokenType::LessThan)), 1),
                });
            } else if eof {
                return Ok((Some((&data[..1], TokenType::LessThan)), 1));
            }, /* else ask more data */
            b'>' => if let Some(b) = data.get(1) {
                return Ok(match *b {
                    b'=' => (Some((&data[..2], TokenType::GreaterEquals)), 2),
                    b'>' => (Some((&data[..2], TokenType::RightShift)), 2),
                    _ => (Some((&data[..1], TokenType::GreaterThan)), 1),
                });
            } else if eof {
                return Ok((Some((&data[..1], TokenType::GreaterThan)), 1));
            }, /* else ask more data */
            b'!' => if let Some(b) = data.get(1) {
                if *b == b'=' {
                    return Ok((Some((&data[..2], TokenType::NotEquals)), 2));
                } else {
                    return Err(Error::ExpectedEqualsSign);
                }
            } else if eof {
                return Err(Error::ExpectedEqualsSign);
            }, /* else ask more data */
            b'|' => if let Some(b) = data.get(1) {
                return Ok(if *b == b'|' {
                    (Some((&data[..2], TokenType::Concat)), 2)
                } else {
                    (Some((&data[..1], TokenType::BitOr)), 1)
                });
            } else if eof {
                return Ok((Some((&data[..1], TokenType::BitOr)), 1));
            }, /* else ask more data */
            b',' => return Ok((Some((&data[..1], TokenType::Comma)), 1)),
            b'&' => return Ok((Some((&data[..1], TokenType::BitAnd)), 1)),
            b'~' => return Ok((Some((&data[..1], TokenType::BitNot)), 1)),
            quote @ b'`' | quote @ b'\'' | quote @ b'"' => return literal(data, eof, quote),
            b'.' => if let Some(b) = data.get(1) {
                if b.is_ascii_digit() {
                    return fractional_part(data, eof);
                } else if eof {
                    return Ok((Some((&data[..1], TokenType::Dot)), 1));
                }
            } else if eof {
                return Ok((Some((&data[..1], TokenType::Dot)), 1));
            }, /* else ask more data */
            b'0'...b'9' => return number(data, eof),
            b'[' => {
                if let Some(i) = memchr(b']', data) {
                    // do not include the '['/']' in the token
                    return Ok((Some((&data[1..i], TokenType::Id)), i));
                } else if eof {
                    return Err(Error::UnterminatedBracket);
                } // else ask more data until ']'
            }
            b'?' => {
                match data.iter().skip(1).position(|&b| !b.is_ascii_digit()) {
                    Some(i) => {
                        // do not include the '?' in the token
                        return Ok((Some((&data[1..i], TokenType::Variable)), i));
                    }
                    None if eof => return Ok((Some((&data[1..], TokenType::Variable)), data.len())),
                    _ => {
                        // else ask more data
                    }
                };
            }
            b'$' | b'@' | b'#' | b':' => {}
            _ => return Err(Error::UnrecognizedToken),
        };
        // Request more data.
        Ok((None, 0))
    }
}

fn literal<'input>(
    data: &'input mut [u8],
    eof: bool,
    quote: u8,
) -> Result<(Option<Token<'input>>, usize), Error> {
    let tt = if quote == b'\'' {
        TokenType::StringLiteral
    } else {
        TokenType::Id
    };
    let mut pb = 0;
    let mut pos = None;
    let mut escaped_quotes = false;
    // data[0] == quote => skip(1)
    for (i, b) in data.iter().enumerate().skip(1) {
        if *b == quote {
            if pb == quote {
                // escaped quote
                pb = 0;
                escaped_quotes = true;
                continue;
            }
        } else if pb == quote {
            pos = Some(i);
            break;
        }
        pb = *b;
    }
    if pos.is_some() || (eof && pb == quote) {
        let i = match pos {
            Some(i) => i,
            _ => data.len(),
        };
        // do not include the quote in the token
        return Ok((
            Some((
                if escaped_quotes {
                    unescape_quotes(&mut data[1..i - 1], quote)
                } else {
                    &data[1..i - 1]
                },
                tt,
            )),
            i,
        ));
    } else if eof {
        return Err(Error::UnterminatedLiteral);
    }
    // else ask more data until closing quote
    Ok((None, 0))
}

fn unescape_quotes(data: &mut [u8], quote: u8) -> &[u8] {
    let mut i = 0;
    let mut j = 0;
    while i < data.len() {
        data[j] = data[i];
        if data[i] == quote {
            i += 1;
        }
        i += 1;
        j += 1;
    }
    &data[..j]
}

fn number<'input>(data: &'input [u8], eof: bool) -> Result<(Option<Token<'input>>, usize), Error> {
    // FIXME
    // else ask more data
    Ok((None, 0))
}

fn fractional_part<'input>(
    data: &'input [u8],
    eof: bool,
) -> Result<(Option<Token<'input>>, usize), Error> {
    // FIXME
    // else ask more data
    Ok((None, 0))
}
