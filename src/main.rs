extern crate mailcheck;



#[cfg(not(test))]
fn parse_msg(path: &Path) -> u32 {
    use std::io::{BufferedReader, File};
    use mailcheck::MessageScanner;

    let file = File::open(path);

    let reader = BufferedReader::new(file);

    let mut parser = MessageScanner::new(reader);

    let mut event_count = 0;

    for _ in parser {
        event_count = event_count + 1;
    }

    event_count
}

#[cfg(not(test))]
fn main() {
    use std::sync::Future;
    use std::io::fs;

    let dir = Path::new("/Users/smckay/projects/rust/mailcheck/msgs");

    match fs::readdir(&dir) {
        Ok(msgs) => {
            let mut events : Vec<Future<u32>> = msgs.iter().map(|msg| {
                let path = msg.clone();
                Future::spawn(proc() { parse_msg(&path) })
            }).collect();
            let msg_count = events.len();
            let event_count = events.iter_mut().map(|e| e.get()).fold(0, |sum, x| sum + x);
            println!("{} events", event_count);
            println!("{} messages", msg_count);
        },
        Err(e) => {
            println!("Error reading directory: {}", e)
        }
    }

}

