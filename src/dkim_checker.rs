extern crate openssl;

use events::MessageParserEvent;
use events::{MessageParserStage,MessageParserFilter};
use events::MessageParserEvent::{Header, BodyChunk};

use self::DkimState::{Start,DkimSignatureSeen,Finished};

use dkim::DkimSignature;
use dkim::DkimVerifier;

pub struct DkimChecker<'a> {
    state: DkimState,
    signatures: Vec<DkimVerifier>,
    next_stage: &'a mut (MessageParserStage + 'a)
}

#[derive(Debug, Clone)]
enum DkimState {
    Start,
    DkimSignatureSeen,
    Finished
}

impl<'a> MessageParserStage for DkimChecker<'a> {

    fn process_event(&mut self, event: MessageParserEvent) {
        let next_state = match self.state {
            Start => self.parse_dkim_headers(event),
            DkimSignatureSeen => self.parse_message(event),
            Finished => {
                self.next_stage.process_event(event);
                Finished
            }
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
            Header(ref name, ref value, _) if *name == dkim_signature_header => {
                //println!("===>  DKIM-Signature: {}", value);
                let signature = DkimSignature::parse(value.as_slice());
                match signature {
                    Ok(s) => {
                        self.signatures.push( DkimVerifier::new(s) );
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
                DkimSignatureSeen
            }
            _ => {
                self.next_stage.process_event(event);
                self.state.clone()
            }
        }
    }

    fn parse_message(&mut self, event: MessageParserEvent) -> DkimState {
        match event {
            BodyChunk(ref data) => {
                for sig in self.signatures.iter_mut() {
                    sig.update_body(data);
                }
                self.next_stage.process_event(event.clone());
                DkimSignatureSeen
            }
            MessageParserEvent::End => {
                loop {
                    let signature = match self.signatures.pop() {
                        Some( sig ) => {
                            sig.finalize_body();
                        }
                        None => break
                    };
                }
                self.next_stage.process_event(event);
                Finished
            }
            _ => {
                self.next_stage.process_event(event);
                DkimSignatureSeen
            }
        }
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
    use std::old_io::MemReader;
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
