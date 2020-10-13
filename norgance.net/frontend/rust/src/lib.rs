#![warn(
    clippy::all,
    //clippy::restriction,
    clippy::pedantic,
    clippy::needless_pass_by_value,
    clippy::unwrap_used,
    clippy::clone_on_ref_ptr
)]
#![allow(
    clippy::clone_on_ref_ptr,
    clippy::else_if_without_else,
    clippy::implicit_return,
    clippy::integer_arithmetic,
    clippy::match_wild_err_arm,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::unreachable,
    clippy::used_underscore_binding,
    clippy::wildcard_imports
)]
mod utils;

use wasm_bindgen::prelude::*;
use snafu::Snafu;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

// The salt is norgance- followed by the first 32 digits of ln(3)
// (prononced Hélène de Troie in French, for Helen of Troy)
// https://en.wikipedia.org/wiki/Nothing-up-my-sleeve_number
const NORGANCE_SALT: &[u8] = b"norgance-1.0986122886681096913952452369225";

#[derive(Debug, Snafu)]
pub enum NorganceError {
    GenericError,
    Argon2Error,
}

impl From<NorganceError> for wasm_bindgen::JsValue {
    fn from(err: NorganceError) -> wasm_bindgen::JsValue {
        JsValue::from_str(&format!("NorganceError: {}", err))
    }
}
//pub type Result<T, E = NorganceError> = std::result::Result<T, E>;

#[wasm_bindgen]
pub fn norgance_identifier(identifier: &str) -> Result<String, JsValue> {
    const ARGON2ID_SETTINGS: argon2::Config = argon2::Config {
        variant: argon2::Variant::Argon2id,
        version: argon2::Version::Version13,
        // These values are low to make it fast enough
        // on slow devices. They are mainly there to make
        // a large scale bruteforce attack a bit more expensive,
        // and also because it's fun.
        // A single blake2b hash could have been good enough.
        // It's designed to take about one second on my laptop.
        mem_cost: 2048, // kb
        time_cost: 3,
        lanes: 1,
        thread_mode: argon2::ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 16,
    };

    let hash = match argon2::hash_raw(identifier.as_bytes(), NORGANCE_SALT, &ARGON2ID_SETTINGS) {
        Ok(hash) => hash,
        Err(_) => return Err(NorganceError::Argon2Error.into())
    };

    let mut bytes: [u8; 16] = [0; 16];
    bytes.clone_from_slice(&hash[0..16]);

    //return Err(NorganceError::GenericError.into());

    Ok(uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::Future)
        .build()
        .to_string())
}

fn norgance_argon2id(identifier: &str, password: &str, mode: &[u8]) -> String {
    const ARGON2ID_SETTINGS: argon2::Config = argon2::Config {
        variant: argon2::Variant::Argon2id,
        version: argon2::Version::Version13,
        mem_cost: 4096,
        time_cost: 3,
        lanes: 1,
        thread_mode: argon2::ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    let salt = [identifier.as_bytes(), &[0x1E], NORGANCE_SALT, &[0x1E], mode].concat();

    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &ARGON2ID_SETTINGS).unwrap();

    return hash;
}

#[wasm_bindgen]
pub fn norgance_citizen_symmetric_key(identifier: &str, password: &str) -> String {
    return norgance_argon2id(identifier, password, b"symmetric_key");
}

#[wasm_bindgen]
pub fn norgance_citizen_access_key(identifier: &str, password: &str) -> String {
    return norgance_argon2id(identifier, password, b"access_key");
}

#[wasm_bindgen]
pub fn norgance_hibp_password_hash(password: &str, size: usize) -> String {
    // The first step is to compute a sha1 of the password
    // because our passwords datasets provides sha1 hashes
    let mut sha1_hasher = sha1::Sha1::new();
    sha1_hasher.update(password.as_bytes());
    let password_sha1 = sha1_hasher.digest().bytes();

    // The next step is about computing a blake2b of the password
    // with some salt specific to norgance. So someone intercepting
    // the request on the network would have to spend quite a lot
    // of ressources to find out what was the origin of the hash

    let hash = blake2_rfc::blake2b::blake2b(size, NORGANCE_SALT, &password_sha1);
    return hex::encode(hash);
}
