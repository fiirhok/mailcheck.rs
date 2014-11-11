use rfc2047::FromRFC2047;
use events::{MessageParserEvent, Header, MessageParserStage};

pub struct HeaderDecoder<T: MessageParserStage> {
    source: Box<T>
}

impl<T: MessageParserStage> HeaderDecoder<T> {
    pub fn new(source: T) -> HeaderDecoder<T> {
        HeaderDecoder { source: box source }
    }

}

impl<T: MessageParserStage> MessageParserStage for HeaderDecoder<T>
{
    fn next(&mut self) -> Option<MessageParserEvent> {
        match self.source.next() {
            Some(Header(name, value)) => {
                Some(Header(name, value.from_rfc2047()))
            },
            event => event
        }
    }
}
