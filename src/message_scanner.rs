use std::vec::Vec;
use std::char::is_whitespace;


use events::MessageParserEvent::{MessageByte,
    HeaderName, HeaderValue, EndOfHeaders, 
    BodyChunk, ParseError, End};
use events::{MessageParserEvent, MessageParserStage, MessageParserFilter};

use self::ParserState::{ParseHeaderName, ParseHeaderValue,
    ParseEndOfHeader, ParseStartHeaderLine, ParseEndOfHeaderSection,
    ParseBody, ParseFinished, ParseStateError};

pub struct MessageScanner<'a> {
    state: ParserState,
    buf: Vec<u8>,
    chunk_size: uint,
    next_stage: &'a mut MessageParserStage + 'a,
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
}

impl<'a> MessageParserFilter<'a> for MessageScanner<'a> {
    fn new(next_stage: &'a mut MessageParserStage) -> MessageScanner<'a> {
        let chunk_size = 2048;
        let buf: Vec<u8> = Vec::with_capacity(chunk_size);
        MessageScanner{ 
            next_stage: next_stage,
            state: ParseHeaderName, 
            buf: buf,
            chunk_size: chunk_size 
        }
    }
}

impl<'a> MessageParserStage for MessageScanner<'a> {
    fn process_event(&mut self, event: MessageParserEvent) {
        let next_state = match event {
            MessageByte(b) => self.process_byte(b),
            End => self.process_end(),
            e => {
                self.next_stage.process_event(e);
                self.state
            }
        };

        self.state = next_state;
    }
}

impl<'a> MessageScanner<'a> {
    fn parse_header_name(&mut self, byte: u8) -> ParserState {

        match byte {
            b':' => match String::from_utf8(self.buf.clone()) {
                    Ok(name) => { 
                        self.buf.clear(); 
                        self.next_stage.process_event(HeaderName(name));
                        ParseHeaderValue
                    },
                    Err(_) => {
                        self.next_stage.process_event(ParseError);
                        ParseStateError
                    }
                },
            _ => { self.buf.push(byte); ParseHeaderName }
        }
    }

    fn parse_header_value(&mut self, byte: u8) -> ParserState {
        match byte {
            b' ' if self.buf.len() == 0 => ParseHeaderValue,
            b'\r' => ParseEndOfHeader,
            b'\n' => ParseStartHeaderLine,
            _ => { self.buf.push(byte); ParseHeaderValue  }

        }
    }

    fn parse_end_of_header(&mut self, byte: u8) -> ParserState {
        match byte {
            b'\n' => ParseStartHeaderLine,
            _ => {
                self.next_stage.process_event(ParseError);
                ParseStateError
            }
        }
    }

    fn parse_start_header_line(&mut self, byte: u8) -> ParserState {
        match byte {
            b'\r' => {
                match String::from_utf8(self.buf.clone()) {
                    Ok(value) => { 
                        self.buf.clear(); 
                        self.next_stage.process_event(HeaderValue(value));
                        ParseEndOfHeaderSection
                    },
                    Err(_) => {
                        self.next_stage.process_event(ParseError);
                        ParseStateError
                    }
                }
            }
            b'\n' => {
                match String::from_utf8(self.buf.clone()) {
                    Ok(value) => { 
                        self.buf.clear(); 
                        self.next_stage.process_event(HeaderValue(value));
                        self.next_stage.process_event(EndOfHeaders);
                        ParseBody
                    },
                    Err(_) => {
                        self.next_stage.process_event(ParseError);
                        ParseStateError
                    }
                }
            }
            x if is_whitespace(x as char) => {
                self.buf.push(x);
                ParseHeaderValue
            },
            _ => match String::from_utf8(self.buf.clone()) {
                Ok(value) => { 
                    self.buf.clear(); 
                    self.buf.push(byte);
                    self.next_stage.process_event(HeaderValue(value));
                    ParseHeaderName
                },
                Err(_) => {
                    self.next_stage.process_event(ParseError);
                    ParseStateError
                }
            },
        }
    }

    fn parse_end_of_header_section(&mut self, byte: u8) -> ParserState {
        match byte {
            b'\n' => {
                self.next_stage.process_event(EndOfHeaders);
                ParseBody
            }
            _ => {
                self.next_stage.process_event(ParseError);
                ParseStateError
            }
        }
    }

    fn parse_body(&mut self, byte: u8) -> ParserState {
        self.buf.push(byte);
        if self.buf.len() < self.chunk_size {
            ParseBody
        }
        else {
            self.next_stage.process_event(BodyChunk(self.buf.clone()));
            self.buf.clear();
            ParseBody
        }
    }

    fn process_byte(&mut self, byte: u8) -> ParserState {
        match self.state {
            ParseFinished => {
                self.next_stage.process_event(End);
                ParseFinished
            }
            ParseStateError => {
                self.next_stage.process_event(End);
                ParseFinished
            }
            ParseHeaderName => self.parse_header_name(byte),
            ParseHeaderValue => self.parse_header_value(byte),
            ParseEndOfHeader => self.parse_end_of_header(byte),
            ParseStartHeaderLine => self.parse_start_header_line(byte),
            ParseEndOfHeaderSection => self.parse_end_of_header_section(byte),
            ParseBody => self.parse_body(byte),
        }
    }

    fn process_end(&mut self) -> ParserState {
        match self.state {
            ParseBody => {
                self.next_stage.process_event(BodyChunk(self.buf.clone()));
                self.buf.clear();
                ParseFinished
            }
            ParseEndOfHeaderSection => ParseFinished,
            _ => {
                self.next_stage.process_event(ParseError);
                ParseStateError
            }
        }
    }
}

#[test]
fn parser_test() {
    let s = "Header1: Value1\r\nHeader2: Value2\r\n\r\nBody".to_string();
    let expected_events = vec![HeaderName("Header1".to_string()), HeaderValue("Value1".to_string()), 
               HeaderName("Header2".to_string()), HeaderValue("Value2".to_string()),
               EndOfHeaders, BodyChunk(vec![66, 111, 100, 121])];

    test_message_scanner(s, expected_events);
}

#[test]
fn multiline_header_test() {
    let s = "Header1: Line1\r\n\t  Line2\r\n\r\nBody".to_string();
    let expected_events = vec![HeaderName("Header1".to_string()), 
        HeaderValue("Line1\t  Line2".to_string()), 
        EndOfHeaders,
        BodyChunk(vec![66, 111, 100, 121])];

    test_message_scanner(s, expected_events);
}

#[cfg(test)]
fn test_message_scanner(msg: String, expected_events: Vec<MessageParserEvent>) {
    use std::io::MemReader;
    use message_parser_sink::MessageParserSink;
    use reader_parser::ReaderParser;


    let mut sink = MessageParserSink::new();
    {
        let r = MemReader::new(msg.as_bytes().to_vec());
        let mut parser: MessageScanner = MessageParserFilter::new(&mut sink);
        let mut rp = ReaderParser::new(&mut parser, r);

        rp.read_to_end();
    }

    assert_eq!(expected_events, sink.events());

}
