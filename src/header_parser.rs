use message_scanner::MessageScanner;
use events::{MessageParserEvent, HeaderName, HeaderValue, Header, 
    EndOfHeaders, ParseError, End};

pub struct HeaderParser<R: Reader> {
    scanner: Box<MessageScanner<R>>,
    state: ParserState,
}

#[deriving(Clone)]
enum ParserState {
    ParseHeaderName,
    ParseHeaderValue(String),
    ParseBody,
    ParseFinished,
    ParseStateError,
    PendingEvents(Box<ParserState>, MessageParserEvent)
}

impl<R: Reader> HeaderParser<R> {
    pub fn new(scanner: MessageScanner<R>) -> HeaderParser<R> {
        HeaderParser{ scanner: box scanner, state: ParseHeaderName }
    }

    fn process_scanner_event(&mut self) -> (ParserState, MessageParserEvent) {
        match self.scanner.next() {
            Some(scanner_event) => match self.state.clone() {
                ParseHeaderName => self.parse_header_name(scanner_event),
                ParseHeaderValue(name) => self.parse_header_value(scanner_event, name),
                ParseBody => (ParseBody, scanner_event),
                _ => (ParseStateError, ParseError)
            },
            None => (ParseFinished, End)
        }
    }

    fn parse_header_name(&mut self, scanner_event: MessageParserEvent) -> (ParserState, MessageParserEvent) {
        match scanner_event.clone() {
            HeaderName(name) => (ParseHeaderValue(name), scanner_event),
            EndOfHeaders => (ParseBody, scanner_event),
            _ => (ParseStateError, ParseError)
        }
    }
    
    fn parse_header_value(&mut self, scanner_event: MessageParserEvent, name: String) -> (ParserState, MessageParserEvent) {
        match scanner_event.clone() {
            HeaderValue(value) => 
                (PendingEvents(box ParseHeaderName, Header(name, value)), scanner_event),
            _ => (ParseStateError, ParseError)
        }
    }
}

impl<R: Reader> Iterator<MessageParserEvent> for HeaderParser<R> {

    fn next(&mut self) -> Option<MessageParserEvent> {

        let (next_state, event) = match self.state {
                PendingEvents(box ref pending_state, ref pending_event) =>
                    (pending_state.clone(), pending_event.clone()),
                _ => self.process_scanner_event()
            };
        
        self.state = next_state;
    
        match event {
            End => return None,
            e => return Some(e)
        }
    }
}


#[test]
fn parser_test() {

    use std::io::MemReader;
    use events::BodyChunk;

    let s = "Header1: Value1\r\nHeader2: Value2\r\n\r\nBody".to_string();

    let r = MemReader::new(s.as_bytes().to_vec());

    let scanner = MessageScanner::new(r);
    let mut parser = HeaderParser::new(scanner);

    let events = parser.collect();

    assert_eq!(vec![HeaderName("Header1".to_string()), HeaderValue("Value1".to_string()), 
               Header("Header1".to_string(), "Value1".to_string()),
               HeaderName("Header2".to_string()), HeaderValue("Value2".to_string()),
               Header("Header2".to_string(), "Value2".to_string()),
               EndOfHeaders, BodyChunk(vec![66, 111, 100, 121])], events);
}
