use events::MessageParserEvent::{HeaderName, HeaderValue, Header, 
    EndOfHeaders, ParseError};
use events::{MessageParserStage, MessageParserFilter};
use events::MessageParserEvent;

use self::ParserState::{ParseHeaderName, ParseHeaderValue, ParseFinished};

pub struct HeaderParser<'a> {
    state: ParserState,
    name: Option<String>,
    next_stage: &'a mut MessageParserStage + 'a
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
            state: ParseHeaderName 
        }
    }
}

impl<'a> HeaderParser<'a> {
    fn parse_header_name(&mut self, event: MessageParserEvent) -> ParserState {
        match event {
            HeaderName(ref name) => {
                self.next_stage.process_event(event.clone());
                self.name = Some(name.clone());
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
                self.next_stage.process_event(event.clone());
                {
                    let name = self.name.clone().expect("ERROR: Header value with no header name");
                    self.next_stage.process_event(Header(name, value.clone()));
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

    let expected_events = vec![HeaderName("Header1".to_string()), 
        HeaderValue("Value1".to_string()), 
        Header("Header1".to_string(), "Value1".to_string()),
        HeaderName("Header2".to_string()), HeaderValue("Value2".to_string()),
        Header("Header2".to_string(), "Value2".to_string()),
        EndOfHeaders, BodyChunk(vec![66, 111, 100, 121])];

    test_message_parser(s, expected_events);
}


#[cfg(test)]
fn test_message_parser(msg: String, expected_events: Vec<MessageParserEvent>) {
    use std::io::MemReader;
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

    assert_eq!(expected_events, sink.events());

}
