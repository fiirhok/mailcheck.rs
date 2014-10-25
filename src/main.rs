#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate mailcheck;

use mailcheck::MessageParser;
use std::io::{BufferedReader, BufferedWriter, File, Truncate, Write};


fn parse_msg() {
    let file = File::open(&Path::new("msgs/msg2"));

    let mut reader = BufferedReader::new(file);

    let mut parser = MessageParser::new(&mut reader);

    for event in parser {
        match event {
            mailcheck::EndOfHeaders => break,
            _ => println!("{}", event)
        }
    }
}

#[cfg(not(test))]
fn main() {
    parse_msg();
}

