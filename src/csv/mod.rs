//! Adaptation/port of
//! [SQLite CSV parser](http://www.sqlite.org/src/artifact?ci=trunk&filename=src/shell.c).
//! See `csv_read_one_field` function in SQLite3 shell sources.
use std::result::Result;

mod error;

pub use error::Error;
pub use scan::Splitter;

/// Reader provides an interface for reading CSV data
/// (compatible with rfc4180 and extended with the option of having a separator other than ",").
/// Successive calls to the `scan` method will step through the 'fields',
/// skipping the separator/newline between the fields.
/// The `EndOfRecord` method tells when a field is terminated by a line break.
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
}

impl Splitter for Reader {
    type E = Error;

    fn split<'input>(
        &mut self,
        data: &'input [u8],
        eof: bool,
    ) -> Result<(Option<&'input [u8]>, usize), Error> {
        if eof && data.is_empty() {
            return Ok((None, 0));
        }
        if !data.is_empty() && data[0] == b'"' {
            // quoted field (may contains separator, newline and escaped quote)
            // Scan until the separator or newline following the closing quote
            // (and ignore escaped quote)
        } else {
            // unquoted field
            // Scan until separator or newline, marking end of field.
        }
        // Request more data.
        Ok((None, 0))
    }
}
