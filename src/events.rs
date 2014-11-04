
#[deriving(Show, PartialEq, Clone)]
pub enum MessageParserEvent {
    HeaderName(String),
    HeaderValue(String),
    EndOfHeaders,
    BodyChunk(Vec<u8>),
    ParseError,
    End,
    NonEvent
}
