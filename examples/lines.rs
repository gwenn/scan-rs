extern crate log;
extern crate scan_rs as scan;

use std::env;
use std::fs::File;
use std::io::{self, Write};

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

use scan::{Liner, Scanner};

fn main() {
    init_logger().unwrap();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let arg = env::args().last().expect("One argument expected");
    println!("{:?}", arg);
    let f = File::open(arg).unwrap();
    let liner = Liner {};
    let mut s = Scanner::new(f, liner);
    loop {
        let field = s.scan().unwrap();
        match field {
            None => break,
            Some((data, _)) => {
                handle.write_all(data).unwrap();
                handle.write(b"\n").unwrap()
            }
        };
    }
}

static LOGGER: Logger = Logger;
struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {
    }
}

fn init_logger() -> Result<(), SetLoggerError> {
    try!(log::set_logger(&LOGGER));
    log::set_max_level(LevelFilter::Debug);
    Ok(())
}
