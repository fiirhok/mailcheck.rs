use rfc2047::FromRFC2047;
use events::MessageParserEvent::Header;
use events::MessageParserEvent;
use events::{MessageParserStage, MessageParserFilter};

pub struct HeaderDecoder<'a> {
    next_stage: &'a mut (MessageParserStage + 'a)
}

impl<'a> MessageParserFilter<'a> for HeaderDecoder<'a> {
    fn new(next_stage: &'a mut MessageParserStage) -> HeaderDecoder<'a> {
        HeaderDecoder { next_stage: next_stage }
    }

}

impl<'a> MessageParserStage for HeaderDecoder<'a>
{
    fn process_event(&mut self, event: MessageParserEvent) {
        match event {
            Header(name, value) => {
                self.next_stage.process_event(Header(name, value.from_rfc2047()))
            },
            _ => self.next_stage.process_event(event)
        }
    }
}
