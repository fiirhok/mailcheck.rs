
#[deriving(Show, PartialEq, Clone)]
pub enum MessageParserEvent {
    HeaderName(String),
    HeaderValue(String),
    Header(String,String),
    EndOfHeaders,
    BodyChunk(Vec<u8>),
    ParseError,
    End,
    NonEvent
}

pub trait MessageParserStage {
    fn next(&mut self) -> Option<MessageParserEvent>;
    fn iter(&mut self) -> MessageParserStageIterator {
        MessageParserStageIterator{ source: self }
    }
}


struct MessageParserStageIterator<'a> {
    source: &'a mut MessageParserStage + 'a
}

impl<'a> Iterator<MessageParserEvent> for MessageParserStageIterator<'a> {
    fn next(&mut self) -> Option<MessageParserEvent> {
        self.source.next()
    }
}

