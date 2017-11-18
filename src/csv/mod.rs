//! Adaptation/port of
//! [SQLite CSV parser](http://www.sqlite.org/src/artifact?ci=trunk&filename=src/shell.c).
//! See `csv_read_one_field` function in SQLite3 shell sources.
use std::ops::Range;
use std::result::Result;
use memchr::memchr2;

mod error;

pub use csv::error::Error;
pub use scan::Splitter;

pub enum FieldType {
    /// Quoted value
    Quoted,
    /// Quoted value with escaped quote
    Escaped,
    /// Not quoted value
    Unquoted,
}

pub type Token<'input> = (&'input [u8], FieldType);

/// Reader provides an interface for reading CSV data
/// (compatible with rfc4180 and extended with the option of having a separator other than ",").
/// Successive calls to the `scan` method will step through the 'fields',
/// skipping the separator/newline between the fields.
/// The `end_of_record` method tells when a field is terminated by a line break.
#[derive(Default)]
pub struct Reader {
    /// values separator
    sep: u8,
    /// specify if values may be quoted (when they contain separator or newline)
    quoted: bool,
    // true when the most recent field has been terminated by a newline (not a separator).
    eor: bool,
    /// trim spaces (only on unquoted values). Break rfc4180 rule: "Spaces are considered part of
    /// a field and should not be ignored."
    pub trim: bool,
    /// character marking the start of a line comment. When specified (not 0), line comment appears
    /// as empty line.
    pub comment: u8,
    /// specify if quoted values may contains unescaped quote not followed by a separator
    /// or a newline
    pub lazy: bool,
    // Index (first is 1) by header
    //pub headers: HashMap<String, u32>
}

impl Reader {
    /// Creates a "standard" CSV reader (separator is comma and quoted mode active)
    pub fn new() -> Reader {
        Reader {
            sep: b',',
            quoted: true,
            eor: true,
            trim: false,
            comment: 0,
            lazy: false,
        }
    }
    /// When `quoted` is `false`, values must not contain a separator or newline.
    pub fn custom(sep: u8, quoted: bool) -> Reader {
        let mut r = Reader::new();
        r.sep = sep;
        r.quoted = quoted;
        r
    }

    pub fn end_of_record(&self) -> bool {
        self.eor
    }
}

impl Splitter for Reader {
    type Error = Error;
    type TokenType = FieldType;

    fn split<'input>(
        &mut self,
        data: &'input mut [u8],
        eof: bool,
    ) -> Result<(Option<Token<'input>>, usize), Error> {
        if eof && data.is_empty() && self.eor {
            return Ok((None, 0));
        }
        if self.quoted && data[0] == b'"' {
            // quoted field (may contains separator, newline and escaped quote)
            return match self.parse_quoted_field(data, eof) {
                Err(e) => Err(e),
                Ok((None, _, n)) => Ok((None, n)),
                Ok((Some(range), true, n)) => Ok((
                    data.get_mut(range)
                        .map(|d| (unescape_quotes(d), FieldType::Escaped)),
                    n,
                )),
                Ok((Some(range), false, n)) => {
                    Ok((data.get(range).map(|d| (d, FieldType::Quoted)), n))
                }
            };
        } else {
            // unquoted field
            // Scan until separator or newline, marking end of field.
            if let Some(i) = memchr2(self.sep, b'\n', data) {
                if data[i] == self.sep {
                    self.eor = false;
                    return Ok((Some((&data[..i], FieldType::Unquoted)), i + 1));
                } else {
                    debug_assert_eq!(data[i], b'\n');
                    self.eor = true;
                    if i > 0 && data[i - 1] == b'\r' {
                        return Ok((Some((&data[..i - 1], FieldType::Unquoted)), i + 1));
                    }
                    return Ok((Some((&data[..i], FieldType::Unquoted)), i + 1));
                }
            }
            // If we're at EOF, we have a final field. Return it.
            if eof {
                self.eor = true;
                return Ok((Some((data, FieldType::Unquoted)), data.len()));
            }
        }
        // Request more data.
        Ok((None, 0))
    }
}

impl Reader {
    fn parse_quoted_field(
        &mut self,
        data: &[u8],
        eof: bool,
    ) -> Result<(Option<Range<usize>>, bool, usize), Error> {
        let iter = data.iter().enumerate().skip(1);
        let mut escaped_quotes = false;
        let mut pb = 0;
        let mut ppb = 0;
        // Scan until the separator or newline following the closing quote
        // (and ignore escaped quote)
        for (i, b) in iter {
            if *b == b'"' && pb == *b {
                pb = 0;
                escaped_quotes = true;
                continue;
            }
            if pb == b'"' && *b == self.sep {
                self.eor = false;
                return Ok((Some(1..i - 1), escaped_quotes, i + 1));
            } else if pb == b'"' && *b == b'\n' {
                self.eor = true;
                return Ok((Some(1..i - 1), escaped_quotes, i + 1));
            } else if ppb == b'"' && pb == b'\r' && *b == b'\n' {
                self.eor = true;
                return Ok((Some(1..i - 2), escaped_quotes, i + 1));
            }
            if pb == b'"' && *b != b'\r' {
                return Err(Error::UnescapedQuote(pb));
            }
            ppb = pb;
            pb = *b;
        }
        if eof {
            if pb == b'\"' {
                self.eor = true;
                let len = data.len();
                return Ok((Some(1..len - 1), escaped_quotes, len));
            }
            // If we're at EOF, we have a non-terminated field.
            return Err(Error::UnterminatedQuotedField);
        }
        // Request more data.
        Ok((None, false, 0))
    }
}

fn unescape_quotes(data: &mut [u8]) -> &[u8] {
    let mut i = 0;
    let mut j = 0;
    while i < data.len() {
        data[j] = data[i];
        if data[i] == b'"' {
            i += 1;
        }
        i += 1;
        j += 1;
    }
    &data[..j]
}
