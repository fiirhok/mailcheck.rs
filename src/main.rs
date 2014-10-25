#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

extern crate mailcheck;

use mailcheck::MessageParser;
use std::io::{BufferedReader, BufferedWriter, File, Truncate, Write};


fn msg_file(index: int) -> File {
    let path = Path::new(format!("msgs/msg{}", index));
    match File::open_mode(&path, Truncate, Write) {
        Ok(file) => file,
        Err(e) => fail!("Error opening message file: {} ({})", path.as_str(), e)
    }
}

fn split_mbox() {
    let file = File::open(&Path::new("Takeout/Mail/All mail Including Spam and Trash.mbox"));
    let mut reader = BufferedReader::new(file);

    let mut i = 0;
    let mut out = BufferedWriter::new(msg_file(i));
    let from_re = regex!(r"^From .*");

    for line in reader.lines() {
        match line {
            Ok(l) => if from_re.is_match(l.as_slice()) {
                    i = i+1;
                    out = BufferedWriter::new(msg_file(i));
                }
                else {
                    out.write_str(l.as_slice());
                },
            _ => fail!("Error reading mbox")
        }

    }

}

fn parse_msg() {
    let file = File::open(&Path::new("Takeout/Mail/msgs/msg.000"));

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
    split_mbox();
}

