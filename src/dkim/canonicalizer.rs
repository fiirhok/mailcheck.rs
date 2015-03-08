use std::ops::Index;
use std::char;

#[derive(Debug,Clone)]
pub enum CanonicalizationType {
    Simple,
    Relaxed
}

pub struct Canonicalizer;

impl Canonicalizer {
    pub fn body(canon_type: CanonicalizationType) -> Box<BodyCanonicalizer + Send> {
        match canon_type {
            CanonicalizationType::Simple => Box::new(SimpleBodyCanonicalizer::new()),
            CanonicalizationType::Relaxed => Box::new(RelaxedBodyCanonicalizer::new())
        }
    }
}

pub trait BodyCanonicalizer {

    fn canonicalize(&mut self, input: &Vec<u8>) -> Vec<u8>;
    fn flush(&mut self) -> Vec<u8>;
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
        for i in range(0, self.pending_newlines ) {
            output.push(b'\r');
            output.push(b'\n');
        }
        self.pending_newlines = 0;

        output.push_all(input.as_slice());

        while (output.len() >= 2 &&
               *output.index(&(output.len() - 1)) == b'\n' &&
               *output.index(&(output.len() - 2)) == b'\r') {

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
        for i in range(0, self.pending_newlines ) {
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


#[test]
fn test_simple_body_canonicalization() {
    use std::vec::as_vec;

    let mut canon = SimpleBodyCanonicalizer::new();

    let mut result = vec![];

    result.push_all(canon.canonicalize(&*as_vec(b"Test\r\nTest \r\n\r\n")).as_slice());
    result.push_all(canon.canonicalize(&*as_vec(b"\r\none  last  line\r\n\r\n")).as_slice());
    result.push_all(canon.flush().as_slice());

    assert_eq!(b"Test\r\nTest \r\n\r\n\r\none  last  line\r\n", result.as_slice());
}

#[test]
fn test_relaxed_body_canonicalization() {
    use std::vec::as_vec;
    use std::str::from_utf8;

    let mut canon = RelaxedBodyCanonicalizer::new();

    let mut result = vec![];

    result.push_all(canon.canonicalize(&*as_vec(b"Test\r\nTest \r\n\r\n")).as_slice());
    result.push_all(canon.canonicalize(&*as_vec(b"\r\none  last \t line\r\n\r\n")).as_slice());
    result.push_all(canon.flush().as_slice());

    assert_eq!("Test\r\nTest\r\n\r\n\r\none last line\r\n", from_utf8(result.as_slice()).unwrap());
}
