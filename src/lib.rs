#![feature(plugin)]
#[plugin] 
extern crate regex_macros;
extern crate regex;

pub use self::events::{MessageParserEvent, MessageParserStage, MessageParserFilter};
pub use self::message_scanner::MessageScanner;
pub use self::header_parser::HeaderParser;
pub use self::header_decoder::HeaderDecoder;
pub use self::rfc2047::FromRFC2047;
pub use self::reader_parser::ReaderParser;
pub use self::message_parser_sink::MessageParserSink;
pub use self::dkim_checker::DkimChecker;

mod events;
mod message_scanner;
mod header_parser;
mod header_decoder;
mod rfc2047;
mod message_parser_sink;
mod reader_parser;
mod dkim_checker;
mod dkim;
