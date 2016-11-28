#![feature(proc_macro)]

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub use std::io::prelude::*;
pub mod lfw;

#[macro_export]
macro_rules! println_stderr (
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);
