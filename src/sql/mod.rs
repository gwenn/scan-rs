//! Adaptation/port of [`SQLite` tokenizer](http://www.sqlite.org/src/artifact?ci=trunk&filename=src/tokenize.c)
use std::result::Result;

use memchr::memchr;
use phf;
use unicase::UniCase;

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

// TODO how to make phf support unicase::Ascii ?
static KEYWORDS: phf::Map<UniCase<&str>, TokenType> = phf_map! {
    UniCase("ABORT") => TokenType::Abort,
    UniCase("ACTION") => TokenType::Action,
    UniCase("ADD") => TokenType::Add,
    UniCase("AFTER") => TokenType::After,
    UniCase("ALL") => TokenType::All,
    UniCase("ALTER") => TokenType::Alter,
    UniCase("ANALYZE") => TokenType::Analyze,
    UniCase("AND") => TokenType::And,
    UniCase("AS") => TokenType::As,
    UniCase("ASC") => TokenType::Asc,
    UniCase("ATTACH") => TokenType::Attach,
    UniCase("AUTOINCREMENT") => TokenType::Autoincr,
    UniCase("BEFORE") => TokenType::Before,
    UniCase("BEGIN") => TokenType::Begin,
    UniCase("BETWEEN") => TokenType::Between,
    UniCase("BY") => TokenType::By,
    UniCase("CASCADE") => TokenType::Cascade,
    UniCase("CASE") => TokenType::Case,
    UniCase("CAST") => TokenType::Cast,
    UniCase("CHECK") => TokenType::Check,
    UniCase("COLLATE") => TokenType::Collate,
    UniCase("COLUMN") => TokenType::ColumnKw,
    UniCase("COMMIT") => TokenType::Commit,
    UniCase("CONFLICT") => TokenType::Conflict,
    UniCase("CONSTRAINT") => TokenType::Constraint,
    UniCase("CREATE") => TokenType::Create,
    UniCase("CROSS") => TokenType::Cross,
    UniCase("CURRENT_DATE") => TokenType::CurrentDate,
    UniCase("CURRENT_TIME") => TokenType::CurrentTime,
    UniCase("CURRENT_TIMESTAMP") => TokenType::CurrentTimestamp,
    UniCase("DATABASE") => TokenType::Database,
    UniCase("DEFAULT") => TokenType::Default,
    UniCase("DEFERRABLE") => TokenType::Deferrable,
    UniCase("DEFERRED") => TokenType::Deferred,
    UniCase("DELETE") => TokenType::Delete,
    UniCase("DESC") => TokenType::Desc,
    UniCase("DETACH") => TokenType::Detach,
    UniCase("DISTINCT") => TokenType::Distinct,
    UniCase("DROP") => TokenType::Drop,
    UniCase("EACH") => TokenType::Each,
    UniCase("ELSE") => TokenType::Else,
    UniCase("END") => TokenType::End,
    UniCase("ESCAPE") => TokenType::Escape,
    UniCase("EXCEPT") => TokenType::Except,
    UniCase("EXCLUSIVE") => TokenType::Exclusive,
    UniCase("EXISTS") => TokenType::Exists,
    UniCase("EXPLAIN") => TokenType::Explain,
    UniCase("FAIL") => TokenType::Fail,
    UniCase("FOR") => TokenType::For,
    UniCase("FOREIGN") => TokenType::Foreign,
    UniCase("FROM") => TokenType::From,
    UniCase("FULL") => TokenType::Full,
    UniCase("GLOB") => TokenType::Glob,
    UniCase("GROUP") => TokenType::Group,
    UniCase("HAVING") => TokenType::Having,
    UniCase("IF") => TokenType::If,
    UniCase("IGNORE") => TokenType::Ignore,
    UniCase("IMMEDIATE") => TokenType::Immediate,
    UniCase("IN") => TokenType::In,
    UniCase("INDEX") => TokenType::Index,
    UniCase("INDEXED") => TokenType::Indexed,
    UniCase("INITIALLY") => TokenType::Initially,
    UniCase("INNER") => TokenType::Inner,
    UniCase("INSERT") => TokenType::Insert,
    UniCase("INSTEAD") => TokenType::Instead,
    UniCase("INTERSECT") => TokenType::Intersect,
    UniCase("INTO") => TokenType::Into,
    UniCase("IS") => TokenType::Is,
    UniCase("ISNULL") => TokenType::IsNull,
    UniCase("JOIN") => TokenType::Join,
    UniCase("KEY") => TokenType::Key,
    UniCase("LEFT") => TokenType::Left,
    UniCase("LIKE") => TokenType::Like,
    UniCase("LIMIT") => TokenType::Limit,
    UniCase("MATCH") => TokenType::Match,
    UniCase("NATURAL") => TokenType::Natural,
    UniCase("NO") => TokenType::No,
    UniCase("NOT") => TokenType::Not,
    UniCase("NOTNULL") => TokenType::NotNull,
    UniCase("NULL") => TokenType::Null,
    UniCase("OF") => TokenType::Of,
    UniCase("OFFSET") => TokenType::Offset,
    UniCase("ON") => TokenType::On,
    UniCase("OR") => TokenType::Or,
    UniCase("ORDER") => TokenType::Order,
    UniCase("OUTER") => TokenType::Outer,
    UniCase("PLAN") => TokenType::Plan,
    UniCase("PRAGMA") => TokenType::Pragma,
    UniCase("PRIMARY") => TokenType::Primary,
    UniCase("QUERY") => TokenType::Query,
    UniCase("RAISE") => TokenType::Raise,
    UniCase("RECURSIVE") => TokenType::Recursive,
    UniCase("REFERENCES") => TokenType::References,
    UniCase("REGEXP") => TokenType::Regexp,
    UniCase("REINDEX") => TokenType::Reindex,
    UniCase("RELEASE") => TokenType::Release,
    UniCase("RENAME") => TokenType::Rename,
    UniCase("REPLACE") => TokenType::Replace,
    UniCase("RESTRICT") => TokenType::Restrict,
    UniCase("RIGHT") => TokenType::Right,
    UniCase("ROLLBACK") => TokenType::Rollback,
    UniCase("ROW") => TokenType::Row,
    UniCase("SAVEPOINT") => TokenType::Savepoint,
    UniCase("SELECT") => TokenType::Select,
    UniCase("SET") => TokenType::Set,
    UniCase("TABLE") => TokenType::Table,
    UniCase("TEMP") => TokenType::Temp,
    UniCase("TEMPORARY") => TokenType::Temp,
    UniCase("THEN") => TokenType::Then,
    UniCase("TO") => TokenType::To,
    UniCase("TRANSACTION") => TokenType::Transaction,
    UniCase("TRIGGER") => TokenType::Trigger,
    UniCase("UNION") => TokenType::Union,
    UniCase("UNIQUE") => TokenType::Unique,
    UniCase("UPDATE") => TokenType::Update,
    UniCase("USING") => TokenType::Using,
    UniCase("VACUUM") => TokenType::Vacuum,
    UniCase("VALUES") => TokenType::Values,
    UniCase("VIEW") => TokenType::View,
    UniCase("VIRTUAL") => TokenType::Virtual,
    UniCase("WHEN") => TokenType::When,
    UniCase("WHERE") => TokenType::Where,
    UniCase("WITH") => TokenType::With,
    UniCase("WITHOUT") => TokenType::Without
};

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
                    Some(i) => i + 1,
                    _ => data.len(),
                },
            ));
        }
        match data[0] {
            b'-' => if let Some(b) = data.get(1) {
                if *b == b'-' {
                    // eat comment
                    if let Some(i) = memchr(b'\n', data) {
                        return Ok((None, i + 1));
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
                    return fractional_part(data, eof, 0);
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
                    return Ok((Some((&data[1..i], TokenType::Id)), i + 1));
                } else if eof {
                    return Err(Error::UnterminatedBracket);
                } // else ask more data until ']'
            }
            b'?' => {
                match data.iter().skip(1).position(|&b| !b.is_ascii_digit()) {
                    Some(i) => {
                        // do not include the '?' in the token
                        return Ok((Some((&data[1..i + 1], TokenType::Variable)), i + 1));
                    }
                    None if eof => return Ok((Some((&data[1..], TokenType::Variable)), data.len())),
                    _ => {
                        // else ask more data
                    }
                };
            }
            b'$' | b'@' | b'#' | b':' => {
                match data.iter()
                    .skip(1)
                    .position(|&b| !is_identifier_continue(b))
                {
                    Some(0) => return Err(Error::BadVariableName),
                    Some(i) => {
                        // '$' is included as part of the name
                        return Ok((Some((&data[..i + 1], TokenType::Variable)), i + 1));
                    }
                    None if eof => return Err(Error::BadVariableName),
                    _ => {
                        // else ask more data
                    }
                };
            }
            b if is_identifier_start(b) => if b == b'x' || b == b'X' {
                if let Some(&b'\'') = data.get(1) {
                    return blob_literal(data, eof);
                } else {
                    return identifierish(data, eof);
                }
            } else {
                return identifierish(data, eof);
            },
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
    debug_assert_eq!(data[0], quote);
    let tt = if quote == b'\'' {
        TokenType::StringLiteral
    } else {
        TokenType::Id
    };
    let mut pb = 0;
    let mut end = None;
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
            end = Some(i);
            break;
        }
        pb = *b;
    }
    if end.is_some() || (eof && pb == quote) {
        let i = match end {
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

fn blob_literal<'input>(
    data: &'input [u8],
    eof: bool,
) -> Result<(Option<Token<'input>>, usize), Error> {
    debug_assert!(data[0] == b'x' || data[0] == b'X');
    debug_assert_eq!(data[1], b'\'');
    if let Some((i, b)) = data.iter()
        .enumerate()
        .skip(2)
        .find(|&(_, &b)| !b.is_ascii_hexdigit())
    {
        if *b != b'\'' || i % 2 != 0 {
            return Err(Error::MalformedBlobLiteral);
        }
        return Ok((Some((&data[2..i], TokenType::Blob)), i + 1));
    } else if eof {
        return Err(Error::MalformedBlobLiteral);
    }
    // else ask more data
    Ok((None, 0))
}

fn number<'input>(data: &'input [u8], eof: bool) -> Result<(Option<Token<'input>>, usize), Error> {
    debug_assert!(data[0].is_ascii_digit());
    if data[0] == b'0' {
        if let Some(b) = data.get(1) {
            if *b == b'x' || *b == b'X' {
                return hex_integer(data, eof);
            }
        } else if eof {
            return Ok((Some((data, TokenType::Integer)), data.len()));
        } else {
            // ask more data
            return Ok((None, 0));
        }
    }
    if let Some((i, b)) = data.iter()
        .enumerate()
        .skip(1)
        .find(|&(_, &b)| !b.is_ascii_digit())
    {
        if *b == b'.' {
            return fractional_part(data, eof, i);
        } else if *b == b'e' || *b == b'E' {
            return exponential_part(data, eof, i);
        } else if is_identifier_start(*b) {
            return Err(Error::BadNumber);
        }
        return Ok((Some((&data[..i], TokenType::Integer)), i));
    } else if eof {
        return Ok((Some((data, TokenType::Integer)), data.len()));
    }
    // else ask more data
    Ok((None, 0))
}

fn hex_integer<'input>(
    data: &'input [u8],
    eof: bool,
) -> Result<(Option<Token<'input>>, usize), Error> {
    debug_assert_eq!(data[0], b'0');
    debug_assert!(data[1] == b'x' || data[1] == b'X');
    if let Some((i, b)) = data.iter()
        .enumerate()
        .skip(2)
        .find(|&(_, &b)| !b.is_ascii_hexdigit())
    {
        // Must not be empty (Ox is invalid)
        if i == 2 || is_identifier_start(*b) {
            return Err(Error::MalformedHexInteger);
        }
        return Ok((Some((&data[..i], TokenType::Integer)), i));
    } else if eof {
        // Must not be empty (Ox is invalid)
        if data.len() == 2 {
            return Err(Error::MalformedHexInteger);
        }
        return Ok((Some((data, TokenType::Integer)), data.len()));
    }
    // else ask more data
    Ok((None, 0))
}

fn fractional_part<'input>(
    data: &'input [u8],
    eof: bool,
    i: usize,
) -> Result<(Option<Token<'input>>, usize), Error> {
    debug_assert_eq!(data[i], b'.');
    if let Some((i, b)) = data.iter()
        .enumerate()
        .skip(i + 1)
        .find(|&(_, &b)| !b.is_ascii_digit())
    {
        if *b == b'e' || *b == b'E' {
            return exponential_part(data, eof, i);
        } else if is_identifier_start(*b) {
            return Err(Error::BadNumber);
        }
        return Ok((Some((&data[..i], TokenType::Float)), i));
    } else if eof {
        return Ok((Some((data, TokenType::Float)), data.len()));
    }
    // else ask more data
    Ok((None, 0))
}

fn exponential_part<'input>(
    data: &'input [u8],
    eof: bool,
    i: usize,
) -> Result<(Option<Token<'input>>, usize), Error> {
    debug_assert!(data[i] == b'e' || data[i] == b'E');
    // data[i] == 'e'|'E'
    if let Some(b) = data.get(i + 1) {
        let i = if *b == b'+' || *b == b'-' { i + 1 } else { i };
        if let Some((i, b)) = data.iter()
            .enumerate()
            .skip(i + 1)
            .find(|&(_, &b)| !b.is_ascii_digit())
        {
            if is_identifier_start(*b) {
                return Err(Error::BadNumber);
            }
            return Ok((Some((&data[..i], TokenType::Float)), i));
        } else if eof {
            return Err(Error::BadNumber);
        }
    } else if eof {
        return Err(Error::BadNumber);
    }
    // else ask more data
    Ok((None, 0))
}

fn identifierish<'input>(
    data: &'input [u8],
    eof: bool,
) -> Result<(Option<Token<'input>>, usize), Error> {
    debug_assert!(is_identifier_start(data[0]));
    // data[0] is_identifier_start => skip(1)
    let end = data.iter()
        .skip(1)
        .position(|&b| !is_identifier_continue(b));
    if end.is_some() || eof {
        let i = match end {
            Some(i) => i + 1,
            _ => data.len(),
        };
        let word = &data[..i];
        let tt = if word.len() > 2 && word.is_ascii() {
            use std::str;
            let s = unsafe { str::from_utf8_unchecked(word) };
            // https://github.com/rust-lang/rust/issues/28853
            let s = unsafe { ::std::mem::transmute::<_, &'static str>(s) };
            KEYWORDS.get(&UniCase(s)).cloned().unwrap_or(TokenType::Id)
        } else {
            TokenType::Id
        };
        return Ok((Some((word, tt)), i));
    }
    // else ask more data
    Ok((None, 0))
}

fn is_identifier_start(b: u8) -> bool {
    (b >= b'A' && b <= b'Z') || b == b'_' || (b >= b'a' && b <= b'z') || b > b'\x7F'
}

fn is_identifier_continue(b: u8) -> bool {
    b == b'$' || (b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'Z') || b == b'_'
        || (b >= b'a' && b <= b'z') || b > b'\x7F'
}
