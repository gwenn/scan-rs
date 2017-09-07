#![feature(read_initializer)]

#[macro_use]
extern crate log;

mod error;
mod scan;

pub use error::Error;
pub use scan::{Result, Scanner};
