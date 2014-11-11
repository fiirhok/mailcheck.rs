extern crate mailcheck;
extern crate time;
use mailcheck::{MessageParserEvent, MessageParserStage};
use mailcheck::Header;

#[cfg(not(test))]
fn parse_msg(path: &Path) -> Box<MessageParserStage> {
    use std::io::{BufferedReader, File};
    use mailcheck::MessageScanner;
    use mailcheck::HeaderParser;
    use mailcheck::HeaderDecoder;

    let file = File::open(path);

    let reader = BufferedReader::new(file);
    let mut scanner = MessageScanner::new(reader);
    let mut parser = HeaderParser::new(scanner);
    box HeaderDecoder::new(parser)
}

#[cfg(not(test))]
fn process_dir(dir: &Path) {
    use std::sync::Future;
    use std::io::fs;

    match fs::readdir(dir) {
        Ok(msgs) => {
            let start = time::precise_time_ns();

            let mut events : Vec<Future<uint>> = msgs.iter().map(|msg| {
                let path = msg.clone();
                Future::spawn(proc() { parse_msg(&path).iter().count() })
            }).collect();
            let msg_count = events.len();
            let event_count = events.iter_mut().map(|e| e.get()).fold(0, |sum, x| sum + x);
            let end = time::precise_time_ns();

            let duration_s = (end - start) as f64 / 1000000000.0;
            let rate = msg_count as f64 / duration_s;
            println!("{} messages in {:.3f} seconds ({:.0f} messages/second)", 
                     msg_count, duration_s, rate);
        },
        Err(e) => {
            println!("Error reading directory: {}", e);
        }
    }
}

#[cfg(not(test))]
fn process_msg(dir: Path, msg: &str) {
    let path = dir.join(Path::new(msg));
    let mut events = parse_msg(&path);

    for event in events.iter() {
        match event {
            Header(name, value) => println!("{}: {}", name, value),
            _ => ()
        }
    }
}

#[cfg(not(test))]
fn main() {
    let mut dir = Path::new("/Users/smckay/projects/rust/mailcheck/msgs");

    //process_dir(&dir);
    process_msg(dir, "msg2");
}

