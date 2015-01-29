use events::MessageParserStage;
use events::MessageParserEvent::{End, MessageByte, ParseError};
use std::old_io::EndOfFile;

pub struct ReaderParser<'a, R: Reader> {
    reader: R,
    next_stage: &'a mut (MessageParserStage + 'a)
}

impl<'a, R: Reader> ReaderParser<'a, R> {

    pub fn new(next_stage: &'a mut MessageParserStage, reader: R) -> ReaderParser<'a, R> {
        ReaderParser {
            reader: reader,
            next_stage: next_stage
        }
    }

    pub fn read_to_end(&mut self) {
        let mut prev_char: u8 = b'\0';
        loop {
            match self.reader.read_byte() {
                Ok(byte) => {
                    if byte == b'\n' && prev_char != b'\r' {
                        self.next_stage.process_event(MessageByte(b'\r'));
                    }
                    prev_char = byte;
                    self.next_stage.process_event(MessageByte(byte))
                },
                Err(e) => {
                    match e.kind {
                        EndOfFile => self.next_stage.process_event(End),
                        _ => self.next_stage.process_event(ParseError)
                    };
                    break
                }
            }
        }
    }
}
