//! Adaptation/port of [`SQLite` tokenizer](http://www.sqlite.org/src/artifact?ci=trunk&filename=src/tokenize.c)
use std::result::Result;

use memchr::memchr;
use phf;

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

static KEYWORDS: phf::Map<&[u8], TokenType> = phf_map! {
    b"ABORT" => TokenType::Abort,
    b"ACTION" => TokenType::Action,
    b"ADD" => TokenType::Add,
    b"AFTER" => TokenType::After,
    b"ALL" => TokenType::All,
    b"ALTER" => TokenType::Alter,
    b"ANALYZE" => TokenType::Analyze,
    b"AND" => TokenType::And,
    b"AS" => TokenType::As,
    b"ASC" => TokenType::Asc,
    b"ATTACH" => TokenType::Attach,
    b"AUTOINCREMENT" => TokenType::Autoincr,
    b"BEFORE" => TokenType::Before,
    b"BEGIN" => TokenType::Begin,
    b"BETWEEN" => TokenType::Between,
    b"BY" => TokenType::By,
    b"CASCADE" => TokenType::Cascade,
    b"CASE" => TokenType::Case,
    b"CAST" => TokenType::Cast,
    b"CHECK" => TokenType::Check,
    b"COLLATE" => TokenType::Collate,
    b"COLUMN" => TokenType::ColumnKw,
    b"COMMIT" => TokenType::Commit,
    b"CONFLICT" => TokenType::Conflict,
    b"CONSTRAINT" => TokenType::Constraint,
    b"CREATE" => TokenType::Create,
    b"CROSS" => TokenType::Cross,
    b"CURRENT_DATE" => TokenType::CurrentDate,
    b"CURRENT_TIME" => TokenType::CurrentTime,
    b"CURRENT_TIMESTAMP" => TokenType::CurrentTimestamp,
    b"DATABASE" => TokenType::Database,
    b"DEFAULT" => TokenType::Default,
    b"DEFERRABLE" => TokenType::Deferrable,
    b"DEFERRED" => TokenType::Deferred,
    b"DELETE" => TokenType::Delete,
    b"DESC" => TokenType::Desc,
    b"DETACH" => TokenType::Detach,
    b"DISTINCT" => TokenType::Distinct,
    b"DROP" => TokenType::Drop,
    b"EACH" => TokenType::Each,
    b"ELSE" => TokenType::Else,
    b"END" => TokenType::End,
    b"ESCAPE" => TokenType::Escape,
    b"EXCEPT" => TokenType::Except,
    b"EXCLUSIVE" => TokenType::Exclusive,
    b"EXISTS" => TokenType::Exists,
    b"EXPLAIN" => TokenType::Explain,
    b"FAIL" => TokenType::Fail,
    b"FOR" => TokenType::For,
    b"FOREIGN" => TokenType::Foreign,
    b"FROM" => TokenType::From,
    b"FULL" => TokenType::Full,
    b"GLOB" => TokenType::Glob,
    b"GROUP" => TokenType::Group,
    b"HAVING" => TokenType::Having,
    b"IF" => TokenType::If,
    b"IGNORE" => TokenType::Ignore,
    b"IMMEDIATE" => TokenType::Immediate,
    b"IN" => TokenType::In,
    b"INDEX" => TokenType::Index,
    b"INDEXED" => TokenType::Indexed,
    b"INITIALLY" => TokenType::Initially,
    b"INNER" => TokenType::Inner,
    b"INSERT" => TokenType::Insert,
    b"INSTEAD" => TokenType::Instead,
    b"INTERSECT" => TokenType::Intersect,
    b"INTO" => TokenType::Into,
    b"IS" => TokenType::Is,
    b"ISNULL" => TokenType::IsNull,
    b"JOIN" => TokenType::Join,
    b"KEY" => TokenType::Key,
    b"LEFT" => TokenType::Left,
    b"LIKE" => TokenType::Like,
    b"LIMIT" => TokenType::Limit,
    b"MATCH" => TokenType::Match,
    b"NATURAL" => TokenType::Natural,
    b"NO" => TokenType::No,
    b"NOT" => TokenType::Not,
    b"NOTNULL" => TokenType::NotNull,
    b"NULL" => TokenType::Null,
    b"OF" => TokenType::Of,
    b"OFFSET" => TokenType::Offset,
    b"ON" => TokenType::On,
    b"OR" => TokenType::Or,
    b"ORDER" => TokenType::Order,
    b"OUTER" => TokenType::Outer,
    b"PLAN" => TokenType::Plan,
    b"PRAGMA" => TokenType::Pragma,
    b"PRIMARY" => TokenType::Primary,
    b"QUERY" => TokenType::Query,
    b"RAISE" => TokenType::Raise,
    b"RECURSIVE" => TokenType::Recursive,
    b"REFERENCES" => TokenType::References,
    b"REGEXP" => TokenType::Regexp,
    b"REINDEX" => TokenType::Reindex,
    b"RELEASE" => TokenType::Release,
    b"RENAME" => TokenType::Rename,
    b"REPLACE" => TokenType::Replace,
    b"RESTRICT" => TokenType::Restrict,
    b"RIGHT" => TokenType::Right,
    b"ROLLBACK" => TokenType::Rollback,
    b"ROW" => TokenType::Row,
    b"SAVEPOINT" => TokenType::Savepoint,
    b"SELECT" => TokenType::Select,
    b"SET" => TokenType::Set,
    b"TABLE" => TokenType::Table,
    b"TEMP" => TokenType::Temp,
    b"TEMPORARY" => TokenType::Temp,
    b"THEN" => TokenType::Then,
    b"TO" => TokenType::To,
    b"TRANSACTION" => TokenType::Transaction,
    b"TRIGGER" => TokenType::Trigger,
    b"UNION" => TokenType::Union,
    b"UNIQUE" => TokenType::Unique,
    b"UPDATE" => TokenType::Update,
    b"USING" => TokenType::Using,
    b"VACUUM" => TokenType::Vacuum,
    b"VALUES" => TokenType::Values,
    b"VIEW" => TokenType::View,
    b"VIRTUAL" => TokenType::Virtual,
    b"WHEN" => TokenType::When,
    b"WHERE" => TokenType::Where,
    b"WITH" => TokenType::With,
    b"WITHOUT" => TokenType::Without
};
const MAX_KEYWORD_LEN: usize = 17;

pub type Token<'input> = (&'input [u8], TokenType);

pub struct Tokenizer {
    uppercase_buffer: [u8; MAX_KEYWORD_LEN],
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer {
            uppercase_buffer: [0; MAX_KEYWORD_LEN],
        }
    }
}

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
                    let mut pb = 0;
                    let mut end = None;
                    for (i, b) in data.iter().enumerate().skip(2) {
                        if *b == b'/' && pb == b'*' {
                            end = Some(i);
                            break;
                        }
                        pb = *b;
                    }
                    if let Some(i) = end {
                        return Ok((None, i + 1));
                    } else if eof {
                        return Err(Error::UnterminatedBlockComment(None));
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
                    return Err(Error::ExpectedEqualsSign(None));
                }
            } else if eof {
                return Err(Error::ExpectedEqualsSign(None));
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
                    return Err(Error::UnterminatedBracket(None));
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
                    Some(0) => return Err(Error::BadVariableName(None)),
                    Some(i) => {
                        // '$' is included as part of the name
                        return Ok((Some((&data[..i + 1], TokenType::Variable)), i + 1));
                    }
                    None if eof => {
                        if data.len() == 1 {
                            return Err(Error::BadVariableName(None));
                        }
                        return Ok((Some((data, TokenType::Variable)), data.len()));
                    }
                    _ => {
                        // else ask more data
                    }
                };
            }
            b if is_identifier_start(b) => if b == b'x' || b == b'X' {
                if let Some(&b'\'') = data.get(1) {
                    return blob_literal(data, eof);
                } else {
                    return self.identifierish(data, eof);
                }
            } else {
                return self.identifierish(data, eof);
            },
            _ => return Err(Error::UnrecognizedToken(None)),
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
        return Err(Error::UnterminatedLiteral(None));
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
            return Err(Error::MalformedBlobLiteral(None));
        }
        return Ok((Some((&data[2..i], TokenType::Blob)), i + 1));
    } else if eof {
        return Err(Error::MalformedBlobLiteral(None));
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
            return Err(Error::BadNumber(None));
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
            return Err(Error::MalformedHexInteger(None));
        }
        return Ok((Some((&data[..i], TokenType::Integer)), i));
    } else if eof {
        // Must not be empty (Ox is invalid)
        if data.len() == 2 {
            return Err(Error::MalformedHexInteger(None));
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
            return Err(Error::BadNumber(None));
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
                return Err(Error::BadNumber(None));
            }
            return Ok((Some((&data[..i], TokenType::Float)), i));
        } else if eof {
            if data.len() == i + 1 {
                return Err(Error::BadNumber(None));
            }
            return Ok((Some((data, TokenType::Float)), data.len()));
        }
    } else if eof {
        return Err(Error::BadNumber(None));
    }
    // else ask more data
    Ok((None, 0))
}

impl Tokenizer {
    fn identifierish<'input>(
        &mut self,
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
            let tt = if word.len() >= 2 && word.len() <= MAX_KEYWORD_LEN && word.is_ascii() {
                let mut buffer = &mut self.uppercase_buffer[..word.len()];
                buffer.copy_from_slice(word);
                buffer.make_ascii_uppercase();
                // https://github.com/rust-lang/rust/issues/28853
                let b = unsafe { ::std::mem::transmute::<_, &'static [u8]>(buffer) };
                KEYWORDS.get(&b).cloned().unwrap_or(TokenType::Id)
            } else {
                TokenType::Id
            };
            return Ok((Some((word, tt)), i));
        }
        // else ask more data
        Ok((None, 0))
    }
}

fn is_identifier_start(b: u8) -> bool {
    (b >= b'A' && b <= b'Z') || b == b'_' || (b >= b'a' && b <= b'z') || b > b'\x7F'
}

fn is_identifier_continue(b: u8) -> bool {
    b == b'$' || (b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'Z') || b == b'_'
        || (b >= b'a' && b <= b'z') || b > b'\x7F'
}
