#![allow(clippy::expect_used)]
extern crate regex;

use regex::Regex;

pub fn key(key: &str) -> bool {
    lazy_static! {
        static ref VALID_KEY: Regex =
            Regex::new("^[a-zA-Z0-9+/]{64}$").expect("Unable to build validate_key regex");
    }
    VALID_KEY.is_match(key)
}

pub fn identifier(identifier: &str) -> bool {
    key(identifier)
}

pub fn curve25519_public_key_base64_no_padding(data: &str) -> bool {
    lazy_static! {
        static ref VALID: Regex =
            Regex::new("^[a-zA-Z0-9+/]{43}$").expect("Unable to build validate_base64 regex");
    }
    VALID.is_match(data)
}

pub fn aead_data_base64_no_padding(data: &str) -> bool {
    lazy_static! {
        static ref VALID: Regex =
            Regex::new("^[a-zA-Z0-9+/]{55,}$").expect("Unable to build validate_base64 regex");
    }
    VALID.is_match(data)
}