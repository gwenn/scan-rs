//! Adaptation/port of
//! [SQLite CSV parser](http://www.sqlite.org/src/artifact?ci=trunk&filename=src/shell.c).
//! See `csv_read_one_field` function in SQLite3 shell sources.
use std::mem;
use std::result::Result;

mod error;

pub use csv::error::Error;
pub use scan::Splitter;

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
    type Item = [u8];

    fn split<'input>(
        &mut self,
        data: &'input mut [u8],
        eof: bool,
    ) -> Result<(Option<&'input [u8]>, usize), Error> {
        if eof && data.is_empty() && self.eor {
            return Ok((None, 0));
        }
        if self.quoted && !data.is_empty() && data[0] == b'"' {
            // quoted field (may contains separator, newline and escaped quote)
            // TODO: I don't know how to make the borrow checker happy!
            let alias: &[u8] = unsafe { mem::transmute(&data[..]) };
            let iter = alias.iter().enumerate().skip(1);
            let mut escaped_quotes = 0;
            let mut pb = 0;
            let mut ppb = 0;
            // Scan until the separator or newline following the closing quote
            // (and ignore escaped quote)
            for (i, b) in iter {
                if *b == b'"' && pb == *b {
                    pb = 0;
                    escaped_quotes += 1;
                    continue;
                }
                if pb == b'"' && *b == self.sep {
                    self.eor = false;
                    return Ok((
                        Some(unescape_quotes(&mut data[1..i - 1], escaped_quotes)),
                        i + 1,
                    ));
                } else if pb == b'"' && *b == b'\n' {
                    self.eor = true;
                    return Ok((
                        Some(unescape_quotes(&mut data[1..i - 1], escaped_quotes)),
                        i + 1,
                    ));
                } else if ppb == b'"' && pb == b'\r' && *b == b'\n' {
                    self.eor = true;
                    return Ok((
                        Some(unescape_quotes(&mut data[1..i - 2], escaped_quotes)),
                        i + 1,
                    ));
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
                    return Ok((
                        Some(unescape_quotes(&mut data[1..len - 1], escaped_quotes)),
                        len,
                    ));
                }
                // If we're at EOF, we have a non-terminated field.
                return Err(Error::UnterminatedQuotedField);
            }
        } else {
            // unquoted field
            let iter = data.iter().enumerate();
            let mut pb = 0;
            // Scan until separator or newline, marking end of field.
            for (i, b) in iter {
                if *b == self.sep {
                    self.eor = false;
                    return Ok((Some(&data[..i]), i + 1));
                } else if *b == b'\n' {
                    self.eor = true;
                    if pb == b'\r' {
                        return Ok((Some(&data[..i - 1]), i + 1));
                    }
                    return Ok((Some(&data[..i]), i + 1));
                }
                pb = *b;
            }
            // If we're at EOF, we have a final field. Return it.
            if eof {
                self.eor = true;
                return Ok((Some(data), data.len()));
            }
        }
        // Request more data.
        Ok((None, 0))
    }
}

fn unescape_quotes(data: &mut [u8], count: usize) -> &[u8] {
    if count == 0 {
        return data;
    }
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
    &data[..data.len() - count]
}
