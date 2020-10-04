#![allow(clippy::result_expect_used)]
extern crate regex;

use regex::Regex;

pub fn validate_key(key: &str) -> bool {
    lazy_static! {
        static ref VALID_KEY: Regex =
            Regex::new("^[a-zA-Z0-9]{64}$").expect("Unable to build validate_key regex");
    }
    VALID_KEY.is_match(key)
}

pub fn validate_identifier(identifier: &str) -> bool {
    validate_key(identifier)
}

pub fn validate_base64(data: &str) -> bool {
    lazy_static! {
        static ref VALID_BASE64: Regex =
            Regex::new("^[a-zA-Z0-9]*$").expect("Unable to build validate_base64 regex");
    }
    VALID_BASE64.is_match(data)
}
