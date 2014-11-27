use std::collections::HashMap;

#[deriving(Show)]
pub struct DkimSignature {
    // REQUIRED:
    version: uint,
    algorithm: String,
    signature: String,
    body_hash: String,
    sdid: String,
    header_fields: Vec<String>,
    selector: String,

    // RECOMMENDED:
    timestamp: Option<uint>,
    expiration: Option<uint>,

    // OPTIONAL:
    canonicalization: Option<String>,
    auid: Option<String>,
    body_length: Option<uint>,
    query_methods: Option<String>,
    copied_header_fields: Option<String>

}

#[deriving(Show)]
pub enum DkimSignatureParseError {
    MissingTag(String),
    BadTag(String)
}

    
impl DkimSignature {
    pub fn parse(signature: &str) -> Result<DkimSignature, DkimSignatureParseError> {


        let tags = try!(parse_dkim_signature(signature));

        Ok(DkimSignature {
            version:  try!(unwrap_uint_tag_value(&tags, "v")),
            algorithm: try!(unwrap_string_tag_value(&tags, "a")),
            signature: try!(unwrap_string_tag_value(&tags, "b")).replace(" ",""),
            body_hash: try!(unwrap_string_tag_value(&tags, "bh")),
            sdid: try!(unwrap_string_tag_value(&tags, "d")),
            header_fields: try!(unwrap_string_tag_value(&tags, "h")).split(':').map(|x| x.to_string()).collect(),
            selector: try!(unwrap_string_tag_value(&tags, "s")),
            timestamp: unwrap_uint_tag_value(&tags, "t").ok(),
            expiration: unwrap_uint_tag_value(&tags, "x").ok(),
            canonicalization: unwrap_string_tag_value(&tags, "c").ok(),
            auid: unwrap_string_tag_value(&tags, "i").ok(),
            body_length: unwrap_uint_tag_value(&tags, "l").ok(),
            query_methods: unwrap_string_tag_value(&tags, "q").ok(),
            copied_header_fields: unwrap_string_tag_value(&tags, "z").ok()
        })
    }
}

fn parse_dkim_signature(dkim_signature: &str) -> Result<HashMap<&str, &str>,DkimSignatureParseError> {
    let mut tags_map : HashMap<&str,&str> = HashMap::new();

    let mut tags = dkim_signature.trim_right_chars(';').split(';');
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

fn unwrap_tag_value<T>(tags: &HashMap<&str,&str>, tag_name: &'static str, transform: |&&str| -> Option<T>) 
    -> Result<T, DkimSignatureParseError> {

    use self::DkimSignatureParseError::MissingTag;

    match tags.get(&tag_name).and_then( transform ) {
        Some(v) => Ok(v),
        None => Err(MissingTag(tag_name.to_string()))
    }
}

fn unwrap_uint_tag_value(tags: &HashMap<&str,&str>, tag_name: &'static str) -> Result<uint,DkimSignatureParseError> {
    unwrap_tag_value(tags, tag_name, |v| from_str(*v))
}

fn unwrap_string_tag_value(tags: &HashMap<&str,&str>, tag_name: &'static str) -> Result<String,DkimSignatureParseError> {
    unwrap_tag_value(tags, tag_name, |v| Some(v.to_string()))
}
