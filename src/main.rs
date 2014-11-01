extern crate mailcheck;

use mailcheck::MessageScanner;
use std::io::{BufferedReader, File, fs};
use std::sync::Future;


#[cfg(not(test))]
fn parse_msg(path: &Path) -> u32 {
    let file = File::open(path);

    let mut reader = BufferedReader::new(file);

    let mut parser = MessageScanner::new(&mut reader);

    let mut event_count = 0;

    for _ in parser {
        event_count = event_count + 1;
    }

    event_count
}

#[cfg(not(test))]
fn main() {

    let dir = Path::new("msgs");

    let mut msg_count: u32 = 0;

    match fs::readdir(&dir) {
        Ok(msgs) => {
            let events = msgs.iter().map(|msg| {
                let path = msg.clone();
                Future::spawn(proc() { parse_msg(&path) })
            });
            let event_count = events.map(|e| e.unwrap()).fold(0, |sum, x| sum + x);
            println!("{} events", event_count);
            println!("{} messages", msg_count);
        },
        Err(e) => {
            println!("Error reading directory: {}", e)
        }
    }

}

