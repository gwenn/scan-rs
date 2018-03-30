#![feature(ascii_ctype)]
#![feature(read_initializer)]
#![feature(plugin)]
#![plugin(phf_macros)]

#[macro_use]
extern crate log;
extern crate memchr;
extern crate phf;

use memchr::memchr;
use std::io::Error;
use std::result::Result;

pub mod csv;
mod scan;
pub mod sql;

pub use scan::{ScanError, Scanner, Splitter};

impl ScanError for Error {
    fn position(&mut self, _: u64, _: usize) {}
}

pub struct Liner {}
pub type Line<'input> = (&'input [u8], ());

impl Splitter for Liner {
    type Error = Error;
    type TokenType = ();

    fn split<'input>(
        &mut self,
        data: &'input mut [u8],
        eof: bool,
    ) -> Result<(Option<Line<'input>>, usize), Error> {
        debug!(target: "scanner", "scan_lines");
        if eof && data.is_empty() {
            return Ok((None, 0));
        }
        if let Some(i) = memchr(b'\n', data) {
            return Ok((Some(drop_cr(&data[..i])), i + 1));
        }
        // If we're at EOF, we have a final, non-terminated line. Return it.
        if eof {
            return Ok((Some(drop_cr(data)), data.len()));
        }
        // Request more data.
        Ok((None, 0))
    }
}

// Drops a terminal \r from the data.
fn drop_cr(data: &[u8]) -> (&[u8], ()) {
    if !data.is_empty() && data[data.len() - 1] == b'\r' {
        return (&data[..data.len() - 1], ());
    }
    (data, ())
}
