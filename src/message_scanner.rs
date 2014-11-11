use std::vec::Vec;
use std::io::EndOfFile;
use std::char::is_whitespace;

use events::{MessageParserEvent, HeaderName, HeaderValue, EndOfHeaders, 
    BodyChunk, ParseError, NonEvent, End, MessageParserStage};

pub struct MessageScanner<R: Reader> {
    reader: R,
    state: ParserState,
    buf: Vec<u8>,
    chunk_size: uint
}

#[deriving(Clone)]
enum ParserState {
    ParseHeaderName,
    ParseHeaderValue,
    ParseEndOfHeader,
    ParseStartHeaderLine,
    ParseEndOfHeaderSection,
    ParseBody, 
    ParseFinished,
    ParseStateError,
    PendingEvents(Box<ParserState>, MessageParserEvent)
}

impl<R: Reader> MessageScanner<R> {
    pub fn new(reader: R) -> MessageScanner<R> {
        let chunk_size = 2048;
        let buf: Vec<u8> = Vec::with_capacity(chunk_size);
        MessageScanner{ reader: reader, 
            state: ParseHeaderName, 
            buf: buf,
            chunk_size: chunk_size }
    }

    fn parse_header_name(&mut self, byte: u8) -> (ParserState, MessageParserEvent) {

        match byte {
            b':' => match String::from_utf8(self.buf.clone()) {
                    Ok(name) => { self.buf.clear(); (ParseHeaderValue, HeaderName(name)) },
                    Err(_) => (ParseStateError, ParseError)
                },
            _ => { self.buf.push(byte); (ParseHeaderName, NonEvent) }
        }
    }

    fn parse_header_value(&mut self, byte: u8) -> (ParserState, MessageParserEvent) {
        match byte {
            b' ' if self.buf.len() == 0 => (ParseHeaderValue, NonEvent),
            b'\r' => (ParseEndOfHeader, NonEvent),
            b'\n' => (ParseStartHeaderLine, NonEvent),
            _ => { self.buf.push(byte); (ParseHeaderValue, NonEvent) }

        }
    }

    fn parse_end_of_header(&mut self, byte: u8) -> (ParserState, MessageParserEvent) {
        match byte {
            b'\n' => (ParseStartHeaderLine, NonEvent),
            _ => (ParseStateError, ParseError)
        }
    }

    fn parse_start_header_line(&mut self, byte: u8) -> (ParserState, MessageParserEvent) {
        match byte {
            b'\r' => {
                match String::from_utf8(self.buf.clone()) {
                    Ok(value) => { 
                        self.buf.clear(); (ParseEndOfHeaderSection, HeaderValue(value)) 
                    },
                    Err(_) => (ParseStateError, ParseError)
                }
            }
            b'\n' => {
                match String::from_utf8(self.buf.clone()) {
                    Ok(value) => { 
                        self.buf.clear(); 
                        (PendingEvents(box ParseBody, EndOfHeaders), HeaderValue(value)) 
                    },
                    Err(_) => (ParseStateError, ParseError)
                }
            }
            x if is_whitespace(x as char) => {
                self.buf.push(x);
                (ParseHeaderValue, NonEvent)
            },
            _ => match String::from_utf8(self.buf.clone()) {
                Ok(value) => { 
                    self.buf.clear(); 
                    self.buf.push(byte);
                    (ParseHeaderName, HeaderValue(value)) 
                },
                Err(_) => (ParseStateError, ParseError)
            },
        }
    }

    fn parse_end_of_header_section(&mut self, byte: u8) -> (ParserState, MessageParserEvent) {
        match byte {
            b'\n' => (ParseBody, EndOfHeaders),
            _ => (ParseStateError, ParseError)
        }
    }

    fn parse_body(&mut self, byte: u8) -> (ParserState, MessageParserEvent) {
        self.buf.push(byte);
        if self.buf.len() < self.chunk_size {
            (ParseBody, NonEvent)
        }
        else {
            let event = BodyChunk(self.buf.clone());
            self.buf.clear();
            (ParseBody, event)
        }
    }

    fn process_byte(&mut self) -> (ParserState, MessageParserEvent) {
        let b = self.reader.read_byte();
        match b {
            Ok(byte) => {
                match self.state {
                    ParseFinished => (ParseFinished, End),
                    ParseStateError => (ParseStateError, End),
                    ParseHeaderName => self.parse_header_name(byte),
                    ParseHeaderValue => self.parse_header_value(byte),
                    ParseEndOfHeader => self.parse_end_of_header(byte),
                    ParseStartHeaderLine => self.parse_start_header_line(byte),
                    ParseEndOfHeaderSection => self.parse_end_of_header_section(byte),
                    ParseBody => self.parse_body(byte),
                    _ => (ParseStateError, ParseError),
                }
            }
            Err(e) => match e.kind {
                EndOfFile => {
                    match self.state {
                        ParseFinished => (ParseFinished, End),
                        _ => {
                            let event = BodyChunk(self.buf.clone());
                            self.buf.clear();
                            (ParseFinished, event)
                        }
                    }
                }
                _ => (ParseStateError, ParseError)
            }
        }
    }
}

impl<R: Reader> MessageParserStage for MessageScanner<R> {

    fn next(&mut self) -> Option<MessageParserEvent> {
        loop {
            let (next_state, event) = match self.state {
                PendingEvents(box ref pending_state, ref pending_event) => 
                    (pending_state.clone(), pending_event.clone()),
                _ => self.process_byte() 
            };

            self.state = next_state;
            match event {
                NonEvent => continue,
                End => return None,
                e => return Some(e)
            }
        }
    }
}

#[test]
fn parser_test() {

    use std::io::MemReader;

    let s = "Header1: Value1\r\nHeader2: Value2\r\n\r\nBody".to_string();

    let r = MemReader::new(s.as_bytes().to_vec());

    let mut parser = MessageScanner::new(r);

    let events = parser.iter().collect();

    assert_eq!(vec![HeaderName("Header1".to_string()), HeaderValue("Value1".to_string()), 
               HeaderName("Header2".to_string()), HeaderValue("Value2".to_string()),
               EndOfHeaders, BodyChunk(vec![66, 111, 100, 121])], events);
}

#[test]
fn multiline_header_test() {

    use std::io::MemReader;

    let s = "Header1: Line1\r\n\t  Line2\r\n\r\nBody".to_string();

    let r = MemReader::new(s.as_bytes().to_vec());

    let mut parser = MessageScanner::new(r);

    let expected_events = [HeaderName("Header1".to_string()), 
        HeaderValue("Line1\t  Line2".to_string()), 
        EndOfHeaders,
        BodyChunk(vec![66, 111, 100, 121])];

    for (expected, actual) in expected_events.iter().zip(parser.iter()) {
        assert_eq!(*expected, actual);
    }
}
