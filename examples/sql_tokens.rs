extern crate scan_rs as scan;

use std::env;
use std::fs::File;
use std::str;
use scan::Scanner;
use scan::sql::Tokenizer;

fn main() {
    let args = env::args();
    for arg in args.skip(1) {
        let f = File::open(arg.clone()).unwrap();
        let tokenizer = Tokenizer {};
        let mut s = Scanner::new(f, tokenizer);
        loop {
            match s.scan() {
                Ok(None) => break,
                Err(err) => {
                    //eprintln!("{} at line: {}, column: {}", err, s.line(), s.column());
                    eprintln!("Err: {}", err);
                    break;
                }
                Ok(Some((token, token_type))) => {
                    println!("'{}', {:?}", str::from_utf8(token).unwrap(), token_type);
                }
            }
        }
    }
}
