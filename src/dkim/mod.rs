extern crate openssl;
extern crate "rustc-serialize" as rustc_serialize;

mod canonicalizer;

use std::collections::HashMap;
use self::openssl::crypto::hash::Hasher;
use self::openssl::crypto::hash::Type;
use self::openssl::crypto::hash::Type::{SHA256,SHA1};

use self::rustc_serialize::base64::{ToBase64,Config};
use self::rustc_serialize::base64::CharacterSet::Standard;
use self::rustc_serialize::base64::Newline::CRLF;

use self::DkimSignatureParseError::BadCanonicalization;
use self::DkimSignatureParseError::MissingTag;


use std::io::Write;

use self::canonicalizer::{CanonicalizationType, Canonicalizer, BodyCanonicalizer};


pub struct DkimSignature {
    // REQUIRED:
    version: u32,
    hash_type: Type,
    signature: String,
    body_hash: String,
    sdid: String,
    header_fields: Vec<String>,
    selector: String,

    // RECOMMENDED:
    timestamp: Option<u32>,
    expiration: Option<u32>,

    // OPTIONAL:
    header_canon: CanonicalizationType,
    body_canon: CanonicalizationType,
    auid: Option<String>,
    body_length: Option<u32>,
    query_methods: Option<String>,
    copied_header_fields: Option<String>
}

#[derive(Debug)]
pub enum DkimSignatureParseError {
    MissingTag(String),
    BadTag(String),
    BadCanonicalization(String),
    BadHashAlgorithm(String),
}

    
impl DkimSignature {
    pub fn parse(signature: &str) -> Result<DkimSignature, DkimSignatureParseError> {
        let tags = try!(parse_dkim_signature(signature));

        let (header_canon, body_canon) = try!(parse_canonicalization(&tags));

        let hash_type = match try!(unwrap_string_tag_value(&tags, "a")).as_slice() {
            "rsa-sha256" => SHA256,
            "rsa-sha1" => SHA1,
            a => return Err(DkimSignatureParseError::BadHashAlgorithm(a.to_string()))
        };

        Ok(DkimSignature {
            version:  try!(unwrap_uint_tag_value(&tags, "v")),
            hash_type: hash_type,
            signature: try!(unwrap_string_tag_value(&tags, "b")).replace(" ",""),
            body_hash: match unwrap_string_tag_value(&tags, "bh") {
                Ok(bh) => regex!(r"\s+").replace_all(bh.as_slice(), "").to_string(),
                Err(e) => return Err(MissingTag("bh".to_string()))
            },            
            sdid: try!(unwrap_string_tag_value(&tags, "d")),
            header_fields: try!(unwrap_string_tag_value(&tags, "h")).split(':').map(|x| x.to_string()).collect(),
            selector: try!(unwrap_string_tag_value(&tags, "s")),
            timestamp: unwrap_uint_tag_value(&tags, "t").ok(),
            expiration: unwrap_uint_tag_value(&tags, "x").ok(),
            header_canon: header_canon,
            body_canon: body_canon,
            auid: unwrap_string_tag_value(&tags, "i").ok(),
            body_length: unwrap_uint_tag_value(&tags, "l").ok(),
            query_methods: unwrap_string_tag_value(&tags, "q").ok(),
            copied_header_fields: unwrap_string_tag_value(&tags, "z").ok()
        })
    }
}


pub struct DkimVerifier {
    signature: DkimSignature,
    hasher: Hasher,
    body_canon: Box<BodyCanonicalizer + Send>,
    header_canon: Box<BodyCanonicalizer + Send>,
    body_bytes_hashed: usize
}

impl DkimVerifier {
    pub fn new(signature: DkimSignature) -> DkimVerifier {
        let hash_type = signature.hash_type;
        let header_canon = signature.header_canon.clone();
        let body_canon = signature.body_canon.clone();
        DkimVerifier {
            signature: signature,
            hasher: Hasher::new(hash_type),
            header_canon: Canonicalizer::body(header_canon),
            body_canon: Canonicalizer::body(body_canon),
            body_bytes_hashed: 0
        }
    }

    fn limit_body_length(&mut self, data: &mut Vec<u8>) {
        match self.signature.body_length {
            Some(body_length) => data.truncate(body_length as usize - self.body_bytes_hashed),
            None => ()
        }
        self.body_bytes_hashed = self.body_bytes_hashed + data.len();
    }

    pub fn update_body(&mut self, data: &Vec<u8>) {
        use std::str::from_utf8;

        let mut canonicalized_data = self.body_canon.canonicalize(data);
        self.limit_body_length(&mut canonicalized_data);
        self.hasher.write(canonicalized_data.as_slice()); 
    }

    pub fn finalize_body(mut self) {
        let mut data = self.body_canon.flush();
        self.limit_body_length(&mut data);
        self.hasher.write(data.as_slice());
        let result = self.hasher.finish();

        let hash_string = result.as_slice().to_base64(Config{
            char_set: Standard, pad: true, newline: CRLF, line_length: None}); 

        if hash_string != self.signature.body_hash {
            let hash_name = match self.signature.hash_type {
                SHA256 => "sha-256",
                SHA1 => "sha-1",
                _ => "err"
            };
            println!("hash mismatch {}", hash_name);
            println!("bh(calc): {}", hash_string);
            println!("bh(sent): {}\n", self.signature.body_hash);
        }

        // TODO: this should return a DkimResults object, so we can actually
        // check the results
    }
}


fn parse_dkim_signature(dkim_signature: &str) -> Result<HashMap<&str, &str>,DkimSignatureParseError> {
    let mut tags_map : HashMap<&str,&str> = HashMap::new();

    let tags = dkim_signature.trim_right_matches(';').split(';');
    for tag in tags {
        let (name, value) = try!(parse_dkim_tag(tag.trim()));
        tags_map.insert(name, value);
    }

    Ok(tags_map)
}

fn parse_dkim_tag(tag: &str) -> Result<(&str, &str),DkimSignatureParseError> {
    use self::DkimSignatureParseError::BadTag;

    let split_tag: Vec<&str> = tag.splitn(1, '=').collect();
    match split_tag.as_slice() {
        [name, value] => Ok((name, value)),
        _ => Err(BadTag(tag.to_string()))
    }
}

fn unwrap_tag_value<T, F>(tags: &HashMap<&str,&str>, tag_name: &'static str, transform: F) 
    -> Result<T, DkimSignatureParseError> 
    where F: Fn(&&str) -> Option<T> 
{

    use self::DkimSignatureParseError::MissingTag;

    match tags.get(&tag_name).and_then( transform ) {
        Some(v) => Ok(v),
        None => Err(MissingTag(tag_name.to_string()))
    }
}

fn unwrap_uint_tag_value(tags: &HashMap<&str,&str>, tag_name: &'static str) -> Result<u32,DkimSignatureParseError> {
    unwrap_tag_value(tags, tag_name, |v| v.parse().ok())
}

fn unwrap_string_tag_value(tags: &HashMap<&str,&str>, tag_name: &'static str) -> Result<String,DkimSignatureParseError> {
    unwrap_tag_value(tags, tag_name, |v| Some(v.to_string()))
}

fn map_canon(s: &str) -> Result<CanonicalizationType, DkimSignatureParseError> {

    match s {
        "simple" => Ok(CanonicalizationType::Simple),
        "relaxed" => Ok(CanonicalizationType::Relaxed),
        _ => Err(BadCanonicalization(s.to_string()))
    }
}

fn parse_canonicalization(tags: &HashMap<&str,&str>) 
    -> Result<(CanonicalizationType, CanonicalizationType), DkimSignatureParseError> {

    let c_regex = regex!(r"(simple|relaxed)(?:/(simple|relaxed))?");

    match unwrap_string_tag_value(tags, "c").ok() {
        None => {
            let header_canon = CanonicalizationType::Simple;
            let body_canon = CanonicalizationType::Simple;
            Ok((header_canon, body_canon))
        },
        Some(c) => {
            match c_regex.captures(c.as_slice()) {
                Some(groups) => match (groups.at(1), groups.at(2)) {
                    (Some(header), Some(body)) => 
                        Ok((try!(map_canon(header)), try!(map_canon(body)))),
                    (Some(header), None) => 
                        Ok((try!(map_canon(header)), CanonicalizationType::Simple)),
                    _ => Err(BadCanonicalization(c.clone())) 
                },
                None => {
                    Err(BadCanonicalization(c.clone()))
                }
            }
        }
    }
}

#[test]
fn test_parse_canonicalization() {
    use self::canonicalizer::CanonicalizationType::Simple;
    use self::canonicalizer::CanonicalizationType::Relaxed;

    fn tags(c_value: &str) -> HashMap<&str,&str> {
        let mut result = HashMap::new();
        result.insert("c", c_value);
        result
    }

    assert!(match parse_canonicalization(&tags("simple/simple")) {
        Ok((Simple,Simple)) => true,
        x => { println!("{:?}", x); false }
    });
    assert!(match parse_canonicalization(&tags("relaxed/relaxed")) {
        Ok((Relaxed,Relaxed)) => true,
        x => { println!("{:?}", x); false }
    });
    assert!(match parse_canonicalization(&tags("simple")) {
        Ok((Simple,Simple)) => true,
        x => { println!("{:?}", x); false }
    });
    assert!(match parse_canonicalization(&tags("relaxed")) {
        Ok((Relaxed,Simple)) => true,
        x => { println!("{:?}", x); false }
    });
    assert!(match parse_canonicalization(&HashMap::new()) {
        Ok((Simple,Simple)) => true,
        x => { println!("{:?}", x); false }
    });
}

