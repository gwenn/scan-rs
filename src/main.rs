#![feature(read_initializer)]

#[macro_use]
extern crate log;

use std::env;
use std::fs::File;
use std::io::{self, Write};

use log::{LogLevel, LogLevelFilter, LogMetadata, LogRecord, SetLoggerError};

mod scan;

use scan::Scanner;

fn main() {
    init_logger().unwrap();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let arg = env::args().last().expect("One argument expected");
    println!("{:?}", arg);
    let f = File::open(arg).unwrap();
    let mut s = Scanner::new(f);
    loop {
        let field = s.scan().unwrap();
        match field {
            None => break,
            Some(data) => handle.write(data).unwrap(),
        };
    }
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            use std::io::Write;
            writeln!(io::stderr(), "{} - {}", record.level(), record.args()).unwrap();
        }
    }
}

fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Debug);
        Box::new(Logger)
    })
}
