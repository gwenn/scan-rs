//! Adaptation/port of [Go scanner](http://tip.golang.org/pkg/bufio/#Scanner).
use std::convert::From;
use std::fmt;
use std::error::Error;
use std::io::{self, BufRead, Read};
use std::result::Result;

pub trait ScanError: Error + From<io::Error> {}

// The arguments are an initial substring of the remaining unprocessed
// data and a flag, `eof`, that reports whether the Reader has no more data
// to give.
//
// If the returned error is non-nil, scanning stops and the error
// is returned to the client.
//
// The function is never called with an empty data slice unless at EOF.
// If `eof` is true, however, data may be non-empty and,
// as always, holds unprocessed text.
pub type FnSplit<E: ScanError> = fn(data: &[u8], eof: bool)
    -> Result<(Option<&[u8]>, usize), E>;

// TODO Result<Option<&[u8]>> or Option<Result<&[u8]>>

// Like a `BufReader` but with a growable buffer.
// Successive calls to the `scan` method will step through the 'tokens'
// of a file, skipping the bytes between the tokens.
//
// Scanning stops unrecoverably at EOF, the first I/O error, or a token too
// large to fit in the buffer. When a scan stops, the reader may have
// advanced arbitrarily far past the last token.
pub struct Scanner<R: Read, E: ScanError> {
    inner: R,          // The reader provided by the client.
    split: FnSplit<E>, // The function to tokenize the input.
    buf: Vec<u8>,      // Buffer used as argument to split.
    pos: usize,        // First non-processed byte in buf.
    cap: usize,        // End of data in buf.
    eof: bool,
}

impl<R: Read, E: ScanError> Scanner<R, E> {
    pub fn new(inner: R, split: FnSplit<E>) -> Scanner<R, E> {
        Self::with_capacity(inner, split, 4096)
    }
    fn with_capacity(inner: R, split: FnSplit<E>, capacity: usize) -> Scanner<R, E> {
        let mut buf = Vec::with_capacity(capacity);
        unsafe {
            buf.set_len(capacity);
            inner.initializer().initialize(&mut buf);
        }
        Scanner {
            inner: inner,
            split: split,
            buf: buf,
            pos: 0,
            cap: 0,
            eof: false,
        }
    }
}

impl<R: Read, E: ScanError> Scanner<R, E> {
    // Advance the Scanner to next token.
    // Return the token as a byte slice.
    // Return `None` when the end of the input is reached.
    // Return any error that occurs while reading the input.
    pub fn scan(&mut self) -> Result<Option<&[u8]>, E> {
        use std::mem;
        debug!(target: "scanner", "scan");
        // Loop until we have a token.
        loop {
            // See if we can get a token with what we already have.
            if self.cap > self.pos || self.eof {
                // TODO: I don't know how to make the borrow checker happy!
                let data = unsafe { mem::transmute(&self.buf[self.pos..self.cap]) };
                match (self.split)(data, self.eof)? {
                    (None, 0) => {}
                    (None, amt) => {
                        self.consume(amt);
                        continue;
                    }
                    (tok, amt) => {
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

impl<R: Read, E: ScanError> BufRead for Scanner<R, E> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        debug!(target: "scanner", "fill_buf");
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

    // consumes `amt` bytes of the buffer.
    fn consume(&mut self, amt: usize) {
        debug!(target: "scanner", "comsume({})", amt);
        assert!(self.pos + amt <= self.cap);
        self.pos += amt;
    }
}

impl<R: Read, E: ScanError> Read for Scanner<R, E> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

impl<R: Read, E: ScanError> fmt::Debug for Scanner<R, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Scanner")
            .field("buf", &self.buf)
            .field("pos", &self.pos)
            .field("cap", &self.cap)
            .field("eof", &self.eof)
            .finish()
    }
}
