extern crate scan_rs;

use std::env;
use std::fs::File;
use scan_rs::Scanner;
use scan_rs::csv::Reader;

fn main() {
    let args = env::args();
    for arg in args.skip(1) {
        let f = File::open(arg.clone()).unwrap();
        let reader = Reader::new();
        let mut s = Scanner::new(f, reader);
        let mut counter = 0;
        loop {
            match s.scan() {
                Ok(None) => break,
                Err(err) => {
                    println!("{} at line: {}, column: {}", err, s.line(), s.column());
                    break;
                }
                Ok(Some(_)) => {
                    counter += 1;
                }
            }
        }
        println!("{}: {}", arg, counter);
    }
}
