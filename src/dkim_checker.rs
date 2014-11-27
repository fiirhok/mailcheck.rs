use events::MessageParserEvent;
use events::{MessageParserStage,MessageParserFilter};
use events::MessageParserEvent::Header;

use self::DkimState::{Start,DkimSignatureSeen};

use dkim::DkimSignature;

pub struct DkimChecker<'a> {
    state: DkimState,
    signatures: Vec<DkimSignature>,
    next_stage: &'a mut MessageParserStage + 'a
}

enum DkimState {
    Start,
    DkimSignatureSeen
}

impl<'a> MessageParserStage for DkimChecker<'a> {

    fn process_event(&mut self, event: MessageParserEvent) {
        let next_state = match self.state {
            Start => self.parse_dkim_headers(event),
            DkimSignatureSeen => self.parse_signed_headers(event)
        };

        self.state = next_state;
    }
}

impl<'a> MessageParserFilter<'a> for DkimChecker<'a> {
    fn new(next_stage: &'a mut MessageParserStage) -> DkimChecker<'a> {
        DkimChecker {
            state: Start,
            signatures: vec![],
            next_stage: next_stage
        }
    }
}

impl<'a> DkimChecker<'a> {
    fn parse_dkim_headers(&mut self, event: MessageParserEvent) -> DkimState {
        let dkim_signature_header = String::from_str("DKIM-Signature");
        match event {
            Header(ref name, ref value) if *name == dkim_signature_header => {
                println!("===>  DKIM-Signature: {}", value);
                let signature = DkimSignature::parse(value.as_slice());
                match signature {
                    Ok(s) => {
                        println!("{}", s);
                        self.signatures.push(s);
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                }
                DkimSignatureSeen
            }
            _ => {
                self.next_stage.process_event(event);
                self.state
            }
        }
    }

    fn parse_signed_headers(&mut self, event: MessageParserEvent) -> DkimState {
        DkimSignatureSeen
    }
}

#[test]
fn parser_test() {
    use events::MessageParserEvent::BodyChunk;

    let s = r"
Header1: Value1
Header2: Value2
DKIM-Signature: test_signature

Body".to_string();

    let expected_events = vec![];

    test_message_parser(s, expected_events);
}

#[cfg(test)]
fn test_message_parser(msg: String, expected_events: Vec<MessageParserEvent>) {
    use std::io::MemReader;
    use message_parser_sink::MessageParserSink;
    use reader_parser::ReaderParser;
    use message_scanner::MessageScanner;
    use header_parser::HeaderParser;
    use events::MessageParserFilter;


    let mut sink = MessageParserSink::new();
    {
        let r = MemReader::new(msg.as_bytes().to_vec());
        let mut dkim: DkimChecker = MessageParserFilter::new(&mut sink);
        let mut parser: HeaderParser = MessageParserFilter::new(&mut dkim);
        let mut scanner: MessageScanner = MessageParserFilter::new(&mut parser);
        let mut rp = ReaderParser::new(&mut scanner, r);

        rp.read_to_end();
    }

    for e in expected_events.iter() {
        assert!(sink.contains(e));
    }

}
