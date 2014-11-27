#![cfg(not(test))]

extern crate mailcheck;
extern crate time;
use mailcheck::MessageParserEvent;

fn parse_msg(path: &Path) -> Vec<MessageParserEvent>
{
    use std::io::{BufferedReader, File};
    use mailcheck::MessageParserFilter;
    use mailcheck::{MessageScanner, HeaderParser, HeaderDecoder, DkimChecker};
    use mailcheck::{ReaderParser, MessageParserSink};


    let file = File::open(path);

    let mut sink = MessageParserSink::new();
    {
        let reader = BufferedReader::new(file);
        let mut dkim_checker: DkimChecker = MessageParserFilter::new(&mut sink);
        let mut header_decoder: HeaderDecoder= MessageParserFilter::new(&mut dkim_checker);
        let mut header_parser: HeaderParser = MessageParserFilter::new(&mut header_decoder);
        let mut message_scanner: MessageScanner = MessageParserFilter::new(&mut header_parser);
        let mut rp = ReaderParser::new(&mut message_scanner, reader);

        rp.read_to_end();
    }
    sink.events()
}

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

fn process_msg(dir: Path, msg: &str) {
    let path = dir.join(Path::new(msg));
    let events = parse_msg(&path);

    for event in events.iter() {
        match event {
            //e => println!("{}", e)
            _ => ()
        }
    }
}

fn main() {
    let dir = Path::new("/Users/smckay/projects/rust/mailcheck/msgs");

    //process_dir(&dir);
    process_msg(dir, "msg2");
}

