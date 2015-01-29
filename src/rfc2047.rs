extern crate regex;
extern crate "rustc-serialize" as rustc_serialize;
extern crate encoding;

use self::regex::{Regex, Captures};
use self::rustc_serialize::base64::FromBase64;
use std::ascii::AsciiExt;
use std::num::FromStrRadix;

use self::encoding::label::encoding_from_whatwg_label;
use self::encoding::DecoderTrap;

use self::FromRFC2047Error::{UnsupportedEncoding, UnsupportedCharset, 
    DecodingError, CharsetError};

#[derive(Show)]
enum FromRFC2047Error {
    UnsupportedEncoding,
    UnsupportedCharset,
    DecodingError,
    CharsetError,
}

pub trait FromRFC2047 {
    fn from_rfc2047(&self) -> String;
}

fn encoded_word_regex() -> Regex {
    let token = r#"[^ "\(\}<>@,;:/\[\]\?\.=]"#; //" (commented quote to help vim syntax highliting)
    let charset_char = token; 
    let encoding_char = token;
    let encoded_char = r"[^\? ]";
    let re_str = format!(r"=\?({}*)\?({}*)\?({}*)\?=",
        charset_char, encoding_char, encoded_char);
    Regex::new(re_str.as_slice()).unwrap()
}

fn decode_word(charset: &str, encoding: &str, content: &str) -> Result<String, FromRFC2047Error> {
    let decoded_content = match encoding.to_ascii_lowercase().as_slice() {
        "q" => q_decode(content),
        "b" => b_decode(content),
        _ => Err(UnsupportedEncoding)
    };
    decoded_content.and_then( 
        |content| charset_decode(charset, content))
}

fn q_decode(content: &str) -> Result<Vec<u8>, FromRFC2047Error> {
    let mut result: Vec<u8> = Vec::new();
    let char_regex = Regex::new(r"(=[0-9a-fA-F]{2}|.)").unwrap();
    for (start,end) in char_regex.find_iter(content) {
        match content.slice(start,end) {
            "_" => result.push(b' '),
            x if x.len() == 3 => {
                let value: u8 = FromStrRadix::from_str_radix(x.slice(1,3), 16).unwrap();
                result.push(value);
            },
            x => result.push_all(x.slice(0,1).as_bytes())
        }
    }
    Ok(result)
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
            match encoding.decode(content.as_slice(), DecoderTrap::Replace) {
                Ok(decoded) => Ok(decoded),
                Err(_) => Err(CharsetError)
            }
        },
        None => Err(UnsupportedCharset)
    }
}

impl FromRFC2047 for str {
    fn from_rfc2047(&self) -> String {
        let encoded_word_re = encoded_word_regex();

        let ws_removed = Regex::new(r"\?=\s*=\?").unwrap().replace_all(self, "?==?");
        encoded_word_re.replace_all(ws_removed.as_slice(), |&: caps: &Captures| {
            match (caps.at(1), caps.at(2), caps.at(3)) {
                (Some(charset), Some(encoding), Some(content)) => {
                    match decode_word(charset, encoding, content) {
                        Ok(decoded) => decoded,
                        Err(_) => self.to_string()
                    }
                }
                _ => self.to_string()
            }
        })
    }
}

#[test]
fn ascii_q_test() {
    let x = "=?utf8?q?test?=";
    assert_eq!("test".to_string(), x.from_rfc2047());
}

#[test]
fn utf8_q_test() {
    let x = "=?utf8?q?test_test=40?=";
    assert_eq!("test test@".to_string(), x.from_rfc2047());
}

#[test]
fn utf8_b_test() {
    let y = "=?UTF8?B?55+l5bex55+l5b2877yM55m+5oiw5LiN5q6G44CC?=";
    assert_eq!("知己知彼，百戰不殆。".to_string(), y.from_rfc2047());
}

#[test]
fn multiword_test() {
    let x = "(=?ISO-8859-1?Q?a?= b)";
    assert_eq!("(a b)".to_string(), x.from_rfc2047());

    let y = "(=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=)";
    assert_eq!("(ab)".to_string(), y.from_rfc2047());
}

#[test]
fn iso_b_multiline_test() {

}

#[test]
fn invalid_test() {
    let x = "test";
    assert_eq!("test".to_string(), x.from_rfc2047());
}

#[test]
fn unsupported_encoding_test() {
    let x = "=?utf8?z?test?=";
    assert_eq!("=?utf8?z?test?=".to_string(), x.from_rfc2047());
}
