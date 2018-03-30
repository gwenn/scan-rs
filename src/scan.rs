//! Adaptation/port of [Go scanner](http://tip.golang.org/pkg/bufio/#Scanner).
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::{self, BufRead, Read};
use std::result::Result;

pub trait ScanError: Error + From<io::Error> + Sized {
    fn position(&mut self, line: u64, column: usize);
}

type SplitResult<'input, TokenType, Error> =
    Result<(Option<(&'input [u8], TokenType)>, usize), Error>;

pub trait Splitter: Sized {
    type Error: ScanError;
    //type Item: ?Sized;
    type TokenType;

    /// The arguments are an initial substring of the remaining unprocessed
    /// data and a flag, `eof`, that reports whether the Reader has no more data
    /// to give.
    ///
    /// If the returned error is non-nil, scanning stops and the error
    /// is returned to the client.
    ///
    /// The function is never called with an empty data slice unless at EOF.
    /// If `eof` is true, however, data may be non-empty and,
    /// as always, holds unprocessed text.
    fn split<'input>(
        &mut self,
        data: &'input mut [u8],
        eof: bool,
    ) -> SplitResult<'input, Self::TokenType, Self::Error>;
}

/// Like a `BufReader` but with a growable buffer.
/// Successive calls to the `scan` method will step through the 'tokens'
/// of a file, skipping the bytes between the tokens.
///
/// Scanning stops unrecoverably at EOF, the first I/O error, or a token too
/// large to fit in the buffer. When a scan stops, the reader may have
/// advanced arbitrarily far past the last token.
pub struct Scanner<R: Read, S: Splitter> {
    /// The reader provided by the client.
    inner: R,
    /// The function to tokenize the input.
    splitter: S,
    /// Buffer used as argument to split.
    buf: Vec<u8>,
    /// First non-processed byte in buf.
    pos: usize,
    /// End of data in buf.
    cap: usize,
    eof: bool,
    /// current line number
    line: u64,
    /// current column number (byte offset, not char offset)
    column: usize,
}

impl<R: Read, S: Splitter> Scanner<R, S> {
    pub fn new(inner: R, splitter: S) -> Scanner<R, S> {
        Self::with_capacity(inner, splitter, 4096)
    }
    fn with_capacity(inner: R, splitter: S, capacity: usize) -> Scanner<R, S> {
        let mut buf = Vec::with_capacity(capacity);
        unsafe {
            buf.set_len(capacity);
            inner.initializer().initialize(&mut buf);
        }
        Scanner {
            inner,
            splitter,
            buf,
            pos: 0,
            cap: 0,
            eof: false,
            line: 1,
            column: 1,
        }
    }

    /// Current line number
    pub fn line(&self) -> u64 {
        self.line
    }

    /// Current column number (byte offset, not char offset)
    pub fn column(&self) -> usize {
        self.column
    }

    pub fn splitter(&self) -> &S {
        &self.splitter
    }

    /// Reset the scanner such that it behaves as if it had never been used.
    pub fn reset(&mut self, inner: R) {
        self.inner = inner;
        self.pos = 0;
        self.cap = 0;
        self.eof = false;
        self.line = 1;
        self.column = 1;
        unsafe {
            self.inner.initializer().initialize(&mut self.buf);
        }
    }
}

type ScanResult<'input, TokenType, Error> = Result<Option<(&'input [u8], TokenType)>, Error>;

impl<R: Read, S: Splitter> Scanner<R, S> {
    /// Advance the Scanner to next token.
    /// Return the token as a byte slice.
    /// Return `None` when the end of the input is reached.
    /// Return any error that occurs while reading the input.
    pub fn scan<'input>(&'input mut self) -> ScanResult<'input, S::TokenType, S::Error> {
        use std::mem;
        debug!(target: "scanner", "scan(line: {}, column: {})", self.line, self.column);
        // Loop until we have a token.
        loop {
            // See if we can get a token with what we already have.
            if self.cap > self.pos || self.eof {
                // TODO: I don't know how to make the borrow checker happy!
                let data = unsafe { mem::transmute(&mut self.buf[self.pos..self.cap]) };
                match self.splitter.split(data, self.eof) {
                    Err(mut e) => {
                        e.position(self.line, self.column);
                        return Err(e);
                    }
                    Ok((None, 0)) => {
                        // Request more data
                    }
                    Ok((None, amt)) => {
                        // Ignore/skip this data
                        self.consume(amt);
                        continue;
                    }
                    Ok((tok, amt)) => {
                        self.consume(amt);
                        return Ok(tok);
                    }
                }
            }
            // We cannot generate a token with what we are holding.
            // If we've already hit EOF, we are done.
            if self.eof {
                // Shut it down.
                self.pos = 0;
                self.cap = 0;
                return Ok(None);
            }
            // Must read more data.
            self.fill_buf()?;
        }
    }
}

impl<R: Read, S: Splitter> BufRead for Scanner<R, S> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        debug!(target: "scanner", "fill_buf: pos: {}, cap: {}, buf: {}", self.pos, self.cap, self.buf.len());
        // First, shift data to beginning of buffer if there's lots of empty space
        // or space is needed.
        if self.pos > 0 && (self.cap == self.buf.len() || self.pos > self.buf.len() / 2) {
            unsafe {
                use std::ptr;
                ptr::copy(
                    self.buf.as_mut_ptr().offset(self.pos as isize),
                    self.buf.as_mut_ptr(),
                    self.cap - self.pos,
                );
            }
            self.cap -= self.pos;
            self.pos = 0
        }
        // Is the buffer full? If so, resize.
        if self.cap == self.buf.len() {
            // TODO maxTokenSize
            let additional = self.buf.capacity();
            self.buf.reserve(additional);
            let cap = self.buf.capacity();
            unsafe {
                self.buf.set_len(cap);
                self.inner
                    .initializer()
                    .initialize(&mut self.buf[self.cap..])
            }
            self.cap -= self.pos;
            self.pos = 0;
        }
        // Finally we can read some input.
        loop {
            match self.inner.read(&mut self.buf[self.cap..]) {
                Ok(0) => {
                    self.eof = true;
                    break;
                }
                Ok(n) => {
                    self.cap += n;
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    /// Consume `amt` bytes of the buffer.
    fn consume(&mut self, amt: usize) {
        debug!(target: "scanner", "comsume({})", amt);
        debug_assert!(self.pos + amt <= self.cap);
        for byte in &self.buf[self.pos..self.pos + amt] {
            if *byte == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.pos += amt;
    }
}

impl<R: Read, S: Splitter> Read for Scanner<R, S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

impl<R: Read, S: Splitter> fmt::Debug for Scanner<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Scanner")
            .field("buf", &self.buf)
            .field("pos", &self.pos)
            .field("cap", &self.cap)
            .field("eof", &self.eof)
            .field("line", &self.line)
            .field("column", &self.column)
            .finish()
    }
}
