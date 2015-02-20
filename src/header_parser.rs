use events::MessageParserEvent::{HeaderName, HeaderValue, Header, 
    EndOfHeaders, ParseError, End};
use events::{MessageParserStage, MessageParserFilter};
use events::MessageParserEvent;

use self::ParserState::{ParseHeaderName, ParseHeaderValue, ParseFinished};

pub struct HeaderParser<'a> {
    state: ParserState,
    name: Option<String>,
    buf: Vec<u8>,
    next_stage: &'a mut (MessageParserStage + 'a)
}

enum ParserState {
    ParseHeaderName,
    ParseHeaderValue,
    ParseFinished,
}

impl<'a> MessageParserStage for HeaderParser<'a> {
    fn process_event(&mut self, event: MessageParserEvent) {
        let next_state = match self.state {
            ParseHeaderName => self.parse_header_name(event),
            ParseHeaderValue => self.parse_header_value(event),
            ParseFinished =>  {
                self.next_stage.process_event(event);
                ParseFinished
            },
        };
        self.state = next_state;
    }
}

impl<'a> MessageParserFilter<'a> for HeaderParser<'a> {
    fn new(next_stage: &'a mut MessageParserStage) -> HeaderParser<'a> {
        HeaderParser{ 
            next_stage: next_stage, 
            name: None,
            buf: vec![],
            state: ParseHeaderName 
        }
    }
}

impl<'a> HeaderParser<'a> {
    fn parse_header_name(&mut self, event: MessageParserEvent) -> ParserState {
        match event {
            HeaderName(ref name) => {
                self.next_stage.process_event(event.clone());
                self.buf.push_all(name.as_bytes());
                let mut trimmed = name.clone();
                // TODO: check to make sure header name ends with ':'
                // but this should always be the case
                trimmed.pop();
                self.name = Some(trimmed);
                ParseHeaderValue
            }
            EndOfHeaders => {
                self.next_stage.process_event(event);
                ParseFinished
            },
            _ => { 
                self.next_stage.process_event(ParseError);
                ParseFinished
            }
        }
    }
    
    fn parse_header_value(&mut self, event: MessageParserEvent) -> ParserState {
        match event {
            HeaderValue(ref value) =>  {
                self.buf.push_all(value.as_bytes());
                self.next_stage.process_event(event.clone());
                {
                    let name = self.name.clone().expect("ERROR: Header value with no header name");
                    let trimmed = value.as_slice().trim().trim_right_matches(':');

                    self.next_stage.process_event(Header(name, trimmed.to_string(), self.buf.clone()));
                    self.buf.clear();
                }
                self.name = None;
                ParseHeaderName
            }
            _ => { 
                self.next_stage.process_event(ParseError);
                ParseFinished
            }
        }
    }
}


#[test]
fn parser_test() {
    use events::MessageParserEvent::BodyChunk;

    let s = "Header1: Value1\r\nHeader2: Value2\r\n\r\nBody".to_string();

    let expected_events = vec![HeaderName("Header1:".to_string()), 
        HeaderValue(" Value1\r\n".to_string()), 
        Header("Header1".to_string(), "Value1".to_string(), "Header1: Value1\r\n".bytes().collect()),
        HeaderName("Header2:".to_string()), 
        HeaderValue(" Value2\r\n".to_string()),
        Header("Header2".to_string(), "Value2".to_string(), "Header2: Value2\r\n".bytes().collect()),
        EndOfHeaders, BodyChunk(vec![66, 111, 100, 121]),End];

    test_message_parser(s, expected_events);
}


#[cfg(test)]
fn test_message_parser(msg: String, expected_events: Vec<MessageParserEvent>) {
    use std::old_io::MemReader;
    use message_parser_sink::MessageParserSink;
    use reader_parser::ReaderParser;
    use message_scanner::MessageScanner;


    let mut sink = MessageParserSink::new();
    {
        let r = MemReader::new(msg.as_bytes().to_vec());
        let mut parser: HeaderParser = MessageParserFilter::new(&mut sink);
        let mut scanner: MessageScanner = MessageParserFilter::new(&mut parser);
        let mut rp = ReaderParser::new(&mut scanner, r);

        rp.read_to_end();
    }

    let actual = sink.events();
    let zipped = expected_events.iter().zip(actual.iter());

    for (expected,actual) in zipped {
        assert_eq!(expected, actual);
    }

}
