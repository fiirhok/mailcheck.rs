extern crate regex;
extern crate serialize;
extern crate collections;
extern crate encoding;

use self::regex::Regex;
use self::serialize::base64::FromBase64;
use std::ascii::AsciiExt;

use self::encoding::label::encoding_from_whatwg_label;
use self::encoding::DecodeStrict;

#[deriving(Show)]
enum FromRFC2047Error {
    UnsupportedEncoding,
    UnsupportedCharset,
    DecodingError,
    CharsetError,
    InvalidEncodedWord,
}

pub trait FromRFC2047 for Sized? {
    fn from_rfc2047(&self) -> Result<String, FromRFC2047Error>;
}

fn encoded_word_regex() -> Regex {
    let token = ".";
    let charset_char = token; 
    let encoding_char = token;
    let encoded_char = token;
    let re_str = format!(r"=\?({}*)\?({}*)\?({}*)\?=",
    charset_char, encoding_char, encoded_char);
    Regex::new(re_str.as_slice()).unwrap()
}

fn decode_word(charset: &str, encoding: &str, content: &str) -> Result<String, FromRFC2047Error> {
    let decoded_content = match encoding.to_ascii_lower().as_slice() {
        "q" => q_decode(content),
        "b" => b_decode(content),
        _ => Err(UnsupportedEncoding)
    };
    decoded_content.and_then( 
        |content| charset_decode(charset, content))
}

fn q_decode(content: &str) -> Result<Vec<u8>, FromRFC2047Error> {
    // TODO: this is junk
    Ok(content.as_bytes().iter().map( |x| *x ).collect())
}

fn b_decode(content: &str) -> Result<Vec<u8>, FromRFC2047Error> {
    match content.from_base64() {
        Ok(x) => Ok(x),
        Err(_) => Err(DecodingError),
    }
}

fn charset_decode(charset: &str, content: Vec<u8>) -> Result<String, FromRFC2047Error> {
    match encoding_from_whatwg_label(charset) {
        Some(encoding) => {
            match encoding.decode(content.as_slice(), DecodeStrict) {
                Ok(decoded) => Ok(decoded),
                Err(_) => Err(CharsetError)
            }
        },
        None => Err(UnsupportedCharset)
    }
}

impl FromRFC2047 for str {
    fn from_rfc2047(&self) -> Result<String, FromRFC2047Error> {
        let encoded_word_re = encoded_word_regex();
        match encoded_word_re.captures(self) {
            Some(groups) => decode_word(groups.at(1), groups.at(2), groups.at(3)),
            None => Err(InvalidEncodedWord)
        }
    }
}

#[test]
fn ascii_q_test() {
    let x = "=?utf8?q?test?=";
    assert_eq!("test".to_string(), x.from_rfc2047().unwrap());
}

#[test]
fn utf8_q_test() {
    let x = "=?utf8?q?test?=";
    assert_eq!("test".to_string(), x.from_rfc2047().unwrap());
}


#[test]
fn utf8_b_test() {
    let y = "=?UTF8?B?55+l5bex55+l5b2877yM55m+5oiw5LiN5q6G44CC?=";
    assert_eq!("知己知彼，百戰不殆。".to_string(), y.from_rfc2047().unwrap());
}

#[test]
fn invalid_test() {
    let x = "test";
    match x.from_rfc2047() {
        Err(InvalidEncodedWord) => (),
        _ => assert!(false)
    }
}

#[test]
fn unsupported_encoding_test() {
    let x = "=?utf8?z?test?=";
    match x.from_rfc2047() {
        Err(UnsupportedEncoding) => (),
        _ => assert!(false)
    }
}
