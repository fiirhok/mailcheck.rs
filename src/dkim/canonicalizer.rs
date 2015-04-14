use std::ops::Index;
use std::char;
use std::ascii::AsciiExt;

#[derive(Debug,Clone)]
pub enum CanonicalizationType {
    Simple,
    Relaxed
}

pub struct Canonicalizer;

impl Canonicalizer {
    pub fn body(canon_type: CanonicalizationType) -> Box<BodyCanonicalizer> {
        match canon_type {
            CanonicalizationType::Simple => Box::new(SimpleBodyCanonicalizer::new()),
            CanonicalizationType::Relaxed => Box::new(RelaxedBodyCanonicalizer::new())
        }
    }
    pub fn head(canon_type: CanonicalizationType) -> Box<HeaderCanonicalizer> {
        match canon_type {
            CanonicalizationType::Simple => Box::new(SimpleHeaderCanonicalizer::new()),
            CanonicalizationType::Relaxed => Box::new(RelaxedHeaderCanonicalizer::new())
        }
    }
}

pub trait BodyCanonicalizer {
    fn canonicalize(&mut self, input: &Vec<u8>) -> Vec<u8>;
    fn flush(&mut self) -> Vec<u8>;
}

pub trait HeaderCanonicalizer {
    fn canonicalize(&mut self, name: String, value: String, raw: Vec<u8>) -> Vec<u8>;
}

struct SimpleBodyCanonicalizer {
    pending_newlines: usize
}

impl SimpleBodyCanonicalizer {
    fn new() -> SimpleBodyCanonicalizer {
        SimpleBodyCanonicalizer { pending_newlines: 0 } 
    }
}

impl BodyCanonicalizer for SimpleBodyCanonicalizer {
    fn canonicalize(&mut self, input: &Vec<u8>) -> Vec<u8> {
        let mut output = vec![];
        for _ in (0 .. self.pending_newlines ) {
            output.push(b'\r');
            output.push(b'\n');
        }
        self.pending_newlines = 0;

        output = output + input;

        while output.len() >= 2 &&
               *output.index(output.len() - 1) == b'\n' &&
               *output.index(output.len() - 2) == b'\r' {

            output.pop();
            output.pop();
            self.pending_newlines = self.pending_newlines + 1;
        }
        output
    }

    fn flush(&mut self) -> Vec<u8> {
        self.pending_newlines = 0;
        vec![b'\r', b'\n']
    }
}

struct RelaxedBodyCanonicalizer {
    pending_newlines: usize,
    ws: bool
}

impl RelaxedBodyCanonicalizer {
    fn new() -> RelaxedBodyCanonicalizer {
        RelaxedBodyCanonicalizer { pending_newlines: 0, ws: false }
    }

    fn flush_newlines(&mut self, output: &mut Vec<u8>) {
        for _ in (0 .. self.pending_newlines ) {
            output.push(b'\r');
            output.push(b'\n');
        }
        self.pending_newlines = 0;
    }

}

impl BodyCanonicalizer for RelaxedBodyCanonicalizer {
    fn canonicalize(&mut self, input: &Vec<u8>) -> Vec<u8> {
        let mut output = vec![];

        for i in input.iter() {
            match char::from_u32(*i as u32) {
                Some(c) => {
                    if c == '\r' {
                        // do nothing
                    }
                    else if c == '\n' {
                        self.ws = false;
                        self.pending_newlines = self.pending_newlines + 1;
                    }
                    else if self.ws {
                        self.flush_newlines(&mut output);
                        if !c.is_whitespace() {
                            output.push(b' ');
                            output.push(*i);
                            self.ws = false;
                        }
                    }
                    else {
                        self.flush_newlines(&mut output);
                        self.ws = c.is_whitespace();
                        if !self.ws {
                            output.push(*i);
                        }
                    }
                }
                None => {
                    // an invalid character is techinically invalid, but we 
                    // don't need to enforce that here:  just pass it through
                    self.flush_newlines(&mut output);
                    output.push(*i);
                    self.ws = false;
                }
            }
        }

        output
    }

    fn flush(&mut self) -> Vec<u8> {
        self.pending_newlines = 0;
        vec![b'\r', b'\n']
    }
}

struct SimpleHeaderCanonicalizer;

impl SimpleHeaderCanonicalizer {
    pub fn new() -> SimpleHeaderCanonicalizer {
        SimpleHeaderCanonicalizer
    }
}

impl HeaderCanonicalizer for SimpleHeaderCanonicalizer {
    fn canonicalize(&mut self, _: String, _: String, raw: Vec<u8>) -> Vec<u8> {
        raw.clone()
    }
}

struct RelaxedHeaderCanonicalizer;

impl RelaxedHeaderCanonicalizer {
    pub fn new() -> RelaxedHeaderCanonicalizer {
        RelaxedHeaderCanonicalizer
    }
}

impl HeaderCanonicalizer for RelaxedHeaderCanonicalizer {
    fn canonicalize(&mut self, name: String, value: String, _: Vec<u8> ) -> Vec<u8> {
        let mut result = name.as_bytes().to_ascii_lowercase();
        result.push(b':');

        let mut ws = false;
        for b in value.as_bytes() {
            let c = match char::from_u32(*b as u32) {
                Some(x) => x,
                None => panic!("Could not decode character")
            };
            if c.is_whitespace() {
                if !ws {
                    ws = true;
                    result.push(b' ');
                }
            }
            else {
                ws = false;
                result.push(*b);
            }
        }
        result + b"\r\n"
    }
}

#[test]
fn test_simple_body_canonicalization() {
    use std::str::from_utf8;

    let mut canon = SimpleBodyCanonicalizer::new();

    let mut result = vec![];

    result.extend(canon.canonicalize(&(Vec::new() + b"Test\r\nTest \r\n\r\n")));
    result.extend(canon.canonicalize(&(Vec::new() + b"\r\none  last  line\r\n\r\n")));
    result.extend(canon.flush());

    assert_eq!("Test\r\nTest \r\n\r\n\r\none  last  line\r\n", from_utf8(&result).unwrap());
}

#[test]
fn test_relaxed_body_canonicalization() {
    use std::str::from_utf8;

    let mut canon = RelaxedBodyCanonicalizer::new();

    let mut result : Vec<u8> = vec![];

    result.extend(canon.canonicalize(&(Vec::new() + b"Test\r\nTest \r\n\r\n")));
    result.extend(canon.canonicalize(&(Vec::new() + b"\r\none  last \t line\r\n\r\n")));
    result.extend(canon.flush());

    assert_eq!("Test\r\nTest\r\n\r\n\r\none last line\r\n", from_utf8(&result[..]).unwrap());
}

#[test]
fn test_simple_header_canonicalization() {
    use std::str::from_utf8;

    let raw = b"Test-Header: Test-Value\r\n   test";
    let name = "Test-Header".to_string();
    let value = "Test-Value\r\n   test".to_string();

    let mut canon = SimpleHeaderCanonicalizer::new();

    let result = canon.canonicalize(name, value, Vec::new() + raw);

    assert_eq!(from_utf8(raw), from_utf8(&result));
}

#[test]
fn test_relaxed_header_canonicalization() {
    use std::str::from_utf8;

    let raw = b"Test-Header: Test-Value\r\n   test";
    let name = "Test-Header".to_string();
    let value = "Test-Value\r\n   test".to_string();

    let mut canon = RelaxedHeaderCanonicalizer::new();

    let result = canon.canonicalize(name, value, Vec::new() + raw);

    assert_eq!(from_utf8(b"test-header:Test-Value test\r\n"), from_utf8(&result));
}
