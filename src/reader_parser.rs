use std::io::Read;

use events::MessageParserStage;
use events::MessageParserEvent::{End, MessageByte, ParseError};


pub struct ReaderParser<'a, R: Read> {
    reader: R,
    next_stage: &'a mut (MessageParserStage + 'a)
}

impl<'a, R: Read> ReaderParser<'a, R> {

    pub fn new(next_stage: &'a mut MessageParserStage, reader: R) -> ReaderParser<'a, R> {
        ReaderParser {
            reader: reader,
            next_stage: next_stage
        }
    }

    pub fn read_to_end(&mut self) {
        const BUF_SIZE: usize = 4 * 1024;
        let mut prev_char: u8 = b'\0';
        loop {
            let mut buf: [u8; BUF_SIZE] = [b'\0'; BUF_SIZE];
            match self.reader.read(&mut buf) {
                Ok(0) => {
                    self.next_stage.process_event(End);
                    break
                },
                Ok(n) => {
                    for i in 0..n {
                        let byte = buf[i];
                        if byte == b'\n' && prev_char != b'\r' {
                            self.next_stage.process_event(MessageByte(b'\r'));
                        }
                        prev_char = byte;
                        self.next_stage.process_event(MessageByte(byte))
                    }
                },
                Err(_) => self.next_stage.process_event(ParseError)
            }
        }
    }
}
