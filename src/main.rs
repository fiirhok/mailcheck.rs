#![cfg(not(test))]

// not worried about using some unstable features here:
#![feature(path)]
#![feature(std_misc)]

extern crate mailcheck;
extern crate time;
use mailcheck::MessageParserEvent;
use std::sync::Future;
use std::fs;
use std::path::{Path,PathBuf};

fn parse_msg(path: &Path) -> Vec<MessageParserEvent>
{
    use std::io::BufReader;
    use std::fs::File;
    use mailcheck::MessageParserFilter;
    use mailcheck::{MessageScanner, HeaderParser, HeaderDecoder, DkimChecker};
    use mailcheck::{ReaderParser, MessageParserSink};

    match File::open(path) {
        Ok(file) => {
            let mut sink = MessageParserSink::new();
            {
                let reader = file; //BufReader::new(file);
                let mut header_decoder: HeaderDecoder= MessageParserFilter::new(&mut sink);
                let mut dkim_checker: DkimChecker = MessageParserFilter::new(&mut header_decoder);
                let mut header_parser: HeaderParser = MessageParserFilter::new(&mut dkim_checker);
                let mut message_scanner: MessageScanner = MessageParserFilter::new(&mut header_parser);
                let mut rp = ReaderParser::new(&mut message_scanner, reader);

                rp.read_to_end();
            }
            sink.events()
        },
        Err(_) => vec!()
    }
}


fn process_msgs_mt(msgs: fs::ReadDir) -> Vec<Future<usize>> {
    msgs.map(|msg| {
        match msg {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                Future::spawn(move || { 
                    parse_msg(&path).iter().count() 
                })
            }
            Err(_) => panic!("Error processing  message")
        }
    }).collect()
}

fn process_msgs(msgs: fs::ReadDir) -> Vec<usize> {
    msgs.map(|msg| {
        match msg {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                parse_msg(&path).iter().count() 
            }
            Err(_) => panic!("Error processing  message")
        }
    }).collect()
}

fn process_dir(dir: &Path) {

    match fs::read_dir(dir) {
        Ok(msgs) => {
            let start = time::precise_time_ns();

            let mut events = process_msgs_mt(msgs);

            let msg_count = events.len();
            let event_count = events.iter_mut().fold(0, |sum, x| sum + x.get());
            let end = time::precise_time_ns();

            let duration_s = (end - start) as f64 / 1000000000.0;
            let rate = msg_count as f64 / duration_s;
            let event_rate = event_count as f64 / duration_s;
            println!("{} messages in {:.3} seconds ({:.0} messages/second)", 
                     msg_count, duration_s, rate);
            println!("{} events in {:.3} seconds ({:.0} events/second)", 
                     event_count, duration_s, event_rate);
        },
        Err(e) => {
            println!("Error reading directory: {}", e);
        }
    }
}

fn process_msg(dir: PathBuf, msg: &str) {
    let path = dir.join(Path::new(msg));
    let events = parse_msg(&path);

    for event in events.iter() {
        match event {
            //e => println!("{:?}", e)
            _ => ()
        }
    }
}

fn main() {
    let dir = PathBuf::new("/Users/smckay/projects/rust/mailcheck/msgs");

    process_dir(&dir);
    //process_msg(dir, "msg10114");
}

