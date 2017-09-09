extern crate scan_rs as scan;

use std::env;
use std::fs::File;
use scan::Scanner;
use scan::csv::Reader;

fn main() {
    let args = env::args();
    for arg in args.skip(1) {
        let f = File::open(arg.clone()).unwrap();
        let reader = Reader::new();
        let mut s = Scanner::new(f, reader);
        let mut counter = 0;
        while let Some(_) = s.scan().unwrap() {
            if s.splitter().end_of_record() {
                counter += 1;
            }
        }
        println!("{}: {}", arg, counter);
    }
}
