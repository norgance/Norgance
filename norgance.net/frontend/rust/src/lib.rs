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

use snafu::Snafu;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// The salt is norgance- followed by the first 32 digits of ln(3)
// (prononced Hélène de Troie in French, for Helen of Troy)
// https://en.wikipedia.org/wiki/Nothing-up-my-sleeve_number
const NORGANCE_SALT: &[u8] = b"norgance-1.0986122886681096913952452369225";

#[derive(Debug, Snafu)]
pub enum NorganceError {
    Argon2,
    ChatrouillePack,
    ChatrouilleUnpack,
    Generic,
    SharedSecret,
    PublicKey,
    InvalidUTF8,
}

impl From<NorganceError> for wasm_bindgen::JsValue {
    fn from(err: NorganceError) -> wasm_bindgen::JsValue {
        JsValue::from_str(&format!("NorganceError: {}", err))
    }
}

pub type Result<T, E = JsValue> = std::result::Result<T, E>;

#[wasm_bindgen]
pub fn norgance_identifier(identifier: &str) -> Result<String> {
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
        Err(_) => return Err(NorganceError::Argon2.into()),
    };

    let mut bytes: [u8; 16] = [0; 16];
    bytes.clone_from_slice(&hash[0..16]);

    Ok(uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::Future)
        .build()
        .to_string())
}

fn norgance_argon2id(identifier: &str, password: &str, mode: &[u8]) -> Result<String> {
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

    match argon2::hash_encoded(password.as_bytes(), &salt, &ARGON2ID_SETTINGS) {
        Ok(hash) => Ok(hash),
        Err(_) => Err(NorganceError::Argon2.into()),
    }
}

#[wasm_bindgen]
pub fn norgance_citizen_symmetric_key(identifier: &str, password: &str) -> Result<String> {
    norgance_argon2id(identifier, password, b"symmetric_key")
}

#[wasm_bindgen]
pub fn norgance_citizen_access_key(identifier: &str, password: &str) -> Result<String> {
    norgance_argon2id(identifier, password, b"access_key")
}

#[must_use]
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
    hex::encode(hash)
}

#[wasm_bindgen]
pub struct ChatrouilleUnsignedQuery {
    query: Vec<u8>,
    shared_secret: Vec<u8>,
}

#[wasm_bindgen]
impl ChatrouilleUnsignedQuery {
    #[must_use]
    pub fn get_query(&self) -> Box<[u8]> {
        self.query.clone().into_boxed_slice()
    }

    #[must_use]
    pub fn get_shared_secret(&mut self) -> Box<[u8]> {
        self.shared_secret.clone().into_boxed_slice()
    }
}

#[wasm_bindgen]
pub fn chatrouille_pack_unsigned_query(payload: &str, public_key: &[u8]) -> Result<ChatrouilleUnsignedQuery> {
    //log!("{:#x?}", public_key);

    let server_public_key = match x448::PublicKey::from_bytes(public_key) {
        Some(pk) => pk,
        None => return Err(NorganceError::PublicKey.into()),
    };

    let (query, shared_secret) =
        match chatrouille::pack_unsigned_query(payload.as_bytes(), &server_public_key) {
            Ok(o) => o,
            Err(_) => return Err(NorganceError::ChatrouillePack.into()),
        };

    Ok(ChatrouilleUnsignedQuery{
        query,
        shared_secret: shared_secret.as_bytes().to_vec(),
    })
}

#[wasm_bindgen]
pub fn chatrouille_unpack_response(packed_data: &[u8], shared_secret: &[u8]) -> Result<String> {
    let shared_secret_instance = match x448::SharedSecret::from_bytes(shared_secret) {
        Some(s) => s,
        None => return Err(NorganceError::SharedSecret.into()),
    };
    let raw_response = match chatrouille::unpack_response(packed_data, &shared_secret_instance) {
        Ok(r) => r,
        Err(_) => return Err(NorganceError::ChatrouilleUnpack.into()),
    };

    match std::str::from_utf8(&raw_response) {
        Ok(r) => Ok(String::from(r)),
        Err(_) => Err(NorganceError::InvalidUTF8.into()),
    }
}