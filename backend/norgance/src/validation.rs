extern crate regex;

use regex::Regex;

pub fn validate_identifier(identifier: &str) -> bool {
    lazy_static! {
        static ref VALID_IDENTIFIER: Regex = Regex::new("^[a-zA-Z0-9]{64}$").unwrap();
    }
    VALID_IDENTIFIER.is_match(identifier)
}