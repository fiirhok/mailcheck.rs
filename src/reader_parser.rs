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
        let mut prev_char: u8 = b'\0';
        loop {
            let mut byte : [u8; 1] = [b'\0'];
            match self.reader.read(&mut byte) {
                Ok(1) => {
                    if byte[0] == b'\n' && prev_char != b'\r' {
                        self.next_stage.process_event(MessageByte(b'\r'));
                    }
                    prev_char = byte[0];
                    self.next_stage.process_event(MessageByte(byte[0]))
                },
                Ok(0) => {
                    self.next_stage.process_event(End)
                },
                Ok(_) => panic!("Unexpected read size"),
                Err(_) => self.next_stage.process_event(ParseError)
            }
        }
    }
}
