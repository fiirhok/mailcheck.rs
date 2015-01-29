
#[derive(Show, PartialEq, Clone)]
pub enum MessageParserEvent {
    MessageByte(u8),
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
    fn process_event(&mut self, event: MessageParserEvent);
}

pub trait MessageParserFilter<'a> : MessageParserStage {
    fn new(next_stage: &'a mut MessageParserStage) -> Self;
}
