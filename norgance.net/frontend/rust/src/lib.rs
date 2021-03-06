#![warn(
    clippy::all,
    // clippy::restriction,
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

use once_cell::sync::OnceCell;
use snafu::{ResultExt, Snafu};
use std::collections::HashMap;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused)]
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
    AccessKeyError,
    Argon2,
    ChatrouilleMissingKeypair,
    ChatrouillePack,
    ChatrouilleUnpack,
    Generic,
    HashError {
        source: std::array::TryFromSliceError,
    },
    InvalidEd25519DalekPrivateKey,
    InvalidEd25519DalekPublicKey,
    InvalidUTF8,
    InvalidX25519DalekPrivateKey,
    InvalidX25519DalekPublicKey,
    InvalidVault,
    NotEnoughEntropy,
    PublicKey,
    PublicKeySignature,
    RandomError {
        source: rand::Error,
    },
    SharedSecret,
    VaultKeyError,
    KeypairError,
    CompressorError {
        source: chatrouille::compressor::CompressorError,
    },
}

impl From<NorganceError> for wasm_bindgen::JsValue {
    fn from(err: NorganceError) -> wasm_bindgen::JsValue {
        JsValue::from_str(&format!("NorganceError: {}", err))
    }
}

pub type Result<T, E = JsValue> = std::result::Result<T, E>;

fn argon2_hash_base64(data: &[u8], salt: &[u8], config: &argon2::Config) -> Result<String> {
    match argon2::hash_raw(data, salt, config) {
        Ok(hash) => Ok(base64::encode_config(hash, base64::STANDARD_NO_PAD)),
        Err(_) => Err(NorganceError::Argon2.into()),
    }
}

#[wasm_bindgen]
pub fn norgance_identifier(identifier: &str) -> Result<String> {
    static CACHE_INSTANCE: OnceCell<Mutex<HashMap<String, String>>> = OnceCell::new();
    let cache_instance = CACHE_INSTANCE.get_or_init(|| Mutex::new(HashMap::new()));

    let cache = cache_instance.lock().unwrap();
    match cache.get(identifier) {
        Some(hash) => return Ok(hash.clone()),
        None => {}
    };
    drop(cache);

    const ARGON2ID_SETTINGS: argon2::Config = argon2::Config {
        variant: argon2::Variant::Argon2id,
        version: argon2::Version::Version13,
        // These values are the default from argon2 officiel implementation.
        // Argon2id is there to make large scale bruteforce attacks
        // a bit more expensive, and also because it's fun.
        mem_cost: 4096,
        time_cost: 3,
        lanes: 1,
        thread_mode: argon2::ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 48, // 48 bytes, 64 bytes long encoded in base64
    };

    match argon2_hash_base64(identifier.as_bytes(), NORGANCE_SALT, &ARGON2ID_SETTINGS) {
        Ok(hash) => {
            let mut cache = cache_instance.lock().unwrap();
            cache.insert(String::from(identifier), hash.clone());
            Ok(hash)
        }
        Err(x) => Err(x),
    }
}

fn norgance_argon2id(identifier: &str, password: &str, mode: &[u8]) -> Result<Vec<u8>> {
    const ARGON2ID_SETTINGS: argon2::Config = argon2::Config {
        variant: argon2::Variant::Argon2id,
        version: argon2::Version::Version13,
        // The memory is a bit more high to make it slower and more difficult
        //mem_cost: 16384,
        mem_cost: 8192,
        time_cost: 3,
        lanes: 1,
        thread_mode: argon2::ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    let salt = [identifier.as_bytes(), &[0x1E], NORGANCE_SALT, &[0x1E], mode].concat();

    match argon2::hash_raw(password.as_bytes(), &salt, &ARGON2ID_SETTINGS) {
        Ok(hash) => Ok(hash),
        Err(_) => Err(NorganceError::Argon2.into()),
    }
}

#[wasm_bindgen]
pub struct NorganceVaultKey {
    key: orion::aead::SecretKey,
}

#[wasm_bindgen]
impl NorganceVaultKey {
    pub fn derive(identifier: &str, password: &str) -> Result<NorganceVaultKey> {
        let raw_key = norgance_argon2id(identifier, password, b"vault_key")?;

        let key = match orion::aead::SecretKey::from_slice(&raw_key) {
            Ok(key) => key,
            Err(_) => return Err(NorganceError::VaultKeyError.into()),
        };

        Ok(NorganceVaultKey { key })
    }
}

#[wasm_bindgen]
pub struct NorganceAccessKey {
    key: NorganceEd25519DalekPrivateKey,
}

#[wasm_bindgen]
impl NorganceAccessKey {
    pub fn derive(identifier: &str, password: &str) -> Result<NorganceAccessKey> {
        let raw_key = norgance_argon2id(identifier, password, b"access_key")?;

        let key = NorganceEd25519DalekPrivateKey::from_bytes(&raw_key)?;

        Ok(NorganceAccessKey { key })
    }

    pub fn get_public_key_base64(&self) -> String {
        self.key.get_public_key().to_base64()
    }
}

impl NorganceAccessKey {
    pub fn to_keypair(&self) -> Result<ed25519_dalek::Keypair> {
        // ed25519_dalek::Keypair do not implement the copy trait
        // So we do it manually from the bytes
        let private_key_bytes = self.key.key.to_bytes();
        let copied_private_key = match ed25519_dalek::SecretKey::from_bytes(&private_key_bytes) {
            Ok(key) => key,
            Err(_) => return Err(NorganceError::KeypairError.into()),
        };

        let secret = copied_private_key;
        let public: ed25519_dalek::PublicKey = (&secret).into();

        Ok(ed25519_dalek::Keypair { public, secret })
    }
}

#[wasm_bindgen]
pub struct NorganceVault {}

#[wasm_bindgen]
impl NorganceVault {
    pub fn open(key: &NorganceVaultKey, encrypted_data_base64: &[u8]) -> Result<String> {
        let encrypted_data = match base64::decode(encrypted_data_base64) {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::InvalidVault.into()),
        };

        let decrypted_data = match orion::aead::open(&key.key, &encrypted_data) {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::InvalidVault.into()),
        };

        let uncompressed_data =
            chatrouille::compressor::decompress(&decrypted_data).context(CompressorError)?;

        match std::str::from_utf8(&uncompressed_data) {
            Ok(r) => Ok(String::from(r)),
            Err(_) => Err(NorganceError::InvalidUTF8.into()),
        }
    }

    pub fn seal(key: &NorganceVaultKey, data_string: &str) -> Result<String> {
        let mut compressed =
            chatrouille::compressor::compress(data_string.as_bytes()).context(CompressorError)?;

        // To slightly improve the privacy, we pad all
        // compressed messages with zeros to have a final size which is a multiple of 32.
        let diff_with_32 = compressed.len() % 32;
        if diff_with_32 != 0 {
            compressed.append(&mut vec![0; 32 - diff_with_32]);
        }

        let encrypted_data = match orion::aead::seal(&key.key, &compressed) {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::InvalidVault.into()),
        };

        let encrypted_data_base64 = base64::encode_config(encrypted_data, base64::STANDARD_NO_PAD);

        Ok(encrypted_data_base64)
    }
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
pub struct Chatrouille {
    server_public_key: x448::PublicKey,
    client_keypair: Option<ed25519_dalek::Keypair>,
}

#[wasm_bindgen]
impl Chatrouille {
    pub fn with_public_key(server_public_key: &[u8]) -> Result<Chatrouille> {
        let server_public_key = match x448::PublicKey::from_bytes(server_public_key) {
            Some(pk) => pk,
            None => return Err(NorganceError::PublicKey.into()),
        };

        Ok(Chatrouille {
            server_public_key,
            client_keypair: None,
        })
    }

    pub fn with_public_key_base64(server_public_key: &str) -> Result<Chatrouille> {
        let key = match base64::decode(server_public_key) {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::PublicKey.into()),
        };

        Chatrouille::with_public_key(&key)
    }

    pub fn with_public_key_and_signature_base64(
        server_public_key: &str,
        signature: &str,
        hardcoded_public_key: &str,
    ) -> Result<Chatrouille> {
        use ed25519_dalek::Verifier;
        use std::convert::TryFrom;
        let server_public_key_bytes = match base64::decode(server_public_key) {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::PublicKey.into()),
        };
        let signature_bytes = match base64::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::PublicKeySignature.into()),
        };
        let signature_instance = match ed25519_dalek::Signature::try_from(&signature_bytes[..]) {
            Ok(signature) => signature,
            Err(_) => return Err(NorganceError::PublicKeySignature.into()),
        };
        let hardcoded_public_key_bytes = match base64::decode(hardcoded_public_key) {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::PublicKeySignature.into()),
        };
        let hardcoded_public_key_instance =
            match ed25519_dalek::PublicKey::from_bytes(&hardcoded_public_key_bytes) {
                Ok(public_key) => public_key,
                Err(_) => return Err(NorganceError::PublicKeySignature.into()),
            };

        match hardcoded_public_key_instance.verify(&server_public_key_bytes, &signature_instance) {
            Ok(_) => (),
            Err(_) => return Err(NorganceError::PublicKeySignature.into()),
        };

        Chatrouille::with_public_key(&server_public_key_bytes)
    }

    pub fn set_client_keypair(&mut self, access_key: &NorganceAccessKey) -> Result<()> {
        let keypair = access_key.to_keypair()?;
        self.client_keypair = Some(keypair);

        Ok(())
    }

    pub fn pack_unsigned_query(&self, payload: &str) -> Result<ChatrouilleQuery> {
        let (query, shared_secret) =
            match chatrouille::pack_unsigned_query(payload.as_bytes(), &self.server_public_key) {
                Ok(o) => o,
                Err(_) => return Err(NorganceError::ChatrouillePack.into()),
            };

        Ok(ChatrouilleQuery {
            query,
            shared_secret,
        })
    }

    pub fn pack_signed_query(&self, payload: &str) -> Result<ChatrouilleQuery> {
        let keypair = match &self.client_keypair {
            Some(keypair) => keypair,
            None => return Err(NorganceError::ChatrouilleMissingKeypair.into()),
        };

        let (query, shared_secret) = match chatrouille::pack_signed_query(
            payload.as_bytes(),
            &self.server_public_key,
            keypair,
        ) {
            Ok(o) => o,
            Err(_) => return Err(NorganceError::ChatrouillePack.into()),
        };

        Ok(ChatrouilleQuery {
            query,
            shared_secret,
        })
    }

    pub fn unpack_response(packed_data: &[u8], query: &ChatrouilleQuery) -> Result<String> {
        let raw_response = match chatrouille::unpack_response(packed_data, &query.shared_secret) {
            Ok(r) => r,
            Err(_) => return Err(NorganceError::ChatrouilleUnpack.into()),
        };
        match std::str::from_utf8(&raw_response) {
            Ok(r) => Ok(String::from(r)),
            Err(_) => Err(NorganceError::InvalidUTF8.into()),
        }
    }
}

#[wasm_bindgen]
pub struct ChatrouilleQuery {
    query: Vec<u8>,
    shared_secret: x448::SharedSecret,
}

#[wasm_bindgen]
impl ChatrouilleQuery {
    #[must_use]
    pub fn get_query(&self) -> Box<[u8]> {
        self.query.clone().into_boxed_slice()
    }
}

#[wasm_bindgen]
pub struct NorganceRng {
    rng: Box<rand::rngs::StdRng>,
}

#[wasm_bindgen]
impl NorganceRng {
    pub fn from_entropy(entropy: &[u8]) -> Result<NorganceRng> {
        use blake2_rfc::blake2b::Blake2b;
        use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
        use std::convert::TryInto;

        if entropy.len() < 1024 {
            return Err(NorganceError::NotEnoughEntropy.into());
        }

        // 1024 CPRNG bits with a seed from crypto.getRandomBytes()
        let mut arr = vec![0_u8; 1024];
        rand::thread_rng()
            .try_fill(&mut arr[..])
            .context(RandomError)?;

        // We combine that with the entropy from the client
        // Using blake2b, because why not ?
        // 256 bits / 32 bytes because it's the max without
        // having to reimplement a rand::rng
        let mut seed_hasher = Blake2b::new(32);
        seed_hasher.update(entropy);
        seed_hasher.update(&arr);

        let seed: [u8; 32] = seed_hasher
            .finalize()
            .as_bytes()
            .try_into()
            .context(HashError)?;

        let mut rng = StdRng::from_seed(seed);

        // Consume 1024 bytes for no good reasons.
        // Only to check that it works, and to make it
        // a bit more difficult to guess the next bytes.
        rng.try_fill_bytes(&mut arr[..]).context(RandomError)?;

        Ok(NorganceRng { rng: Box::new(rng) })
    }
}

#[wasm_bindgen]
pub struct NorganceX25519DalekPrivateKey {
    key: x25519_dalek::StaticSecret,
}

#[wasm_bindgen]
impl NorganceX25519DalekPrivateKey {
    pub fn from_rng(rng: &mut NorganceRng) -> NorganceX25519DalekPrivateKey {
        let key = x25519_dalek::StaticSecret::new(&mut rng.rng);
        NorganceX25519DalekPrivateKey { key }
    }

    pub fn from_base64(private_key_base64: &str) -> Result<NorganceX25519DalekPrivateKey> {
        use std::convert::TryInto;

        let bytes = match base64::decode(private_key_base64) {
            Ok(bytes) => bytes.into_boxed_slice(),
            Err(_) => return Err(NorganceError::InvalidX25519DalekPrivateKey.into()),
        };

        let bytes: Box<[u8; 32]> = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::InvalidX25519DalekPrivateKey.into()),
        };

        let key = x25519_dalek::StaticSecret::from(*bytes);
        Ok(NorganceX25519DalekPrivateKey { key })
    }

    #[must_use]
    pub fn to_base64(&self) -> String {
        base64::encode_config(&self.key.to_bytes().to_vec(), base64::STANDARD_NO_PAD)
    }

    #[must_use]
    pub fn get_public_key(&self) -> NorganceX25519DalekPublicKey {
        let key = x25519_dalek::PublicKey::from(&self.key);
        NorganceX25519DalekPublicKey { key }
    }
}

#[wasm_bindgen]
pub struct NorganceX25519DalekPublicKey {
    key: x25519_dalek::PublicKey,
}

#[wasm_bindgen]
impl NorganceX25519DalekPublicKey {
    pub fn from_base64(public_key_base64: &str) -> Result<NorganceX25519DalekPublicKey> {
        use std::convert::TryInto;

        let bytes = match base64::decode(public_key_base64) {
            Ok(bytes) => bytes.into_boxed_slice(),
            Err(_) => return Err(NorganceError::InvalidX25519DalekPublicKey.into()),
        };

        let bytes: Box<[u8; 32]> = match bytes.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(NorganceError::InvalidX25519DalekPublicKey.into()),
        };

        let key = x25519_dalek::PublicKey::from(*bytes);
        Ok(NorganceX25519DalekPublicKey { key })
    }

    #[must_use]
    pub fn to_base64(&self) -> String {
        base64::encode_config(self.key.as_bytes().to_vec(), base64::STANDARD_NO_PAD)
    }
}

#[wasm_bindgen]
pub struct NorganceEd25519DalekPrivateKey {
    key: ed25519_dalek::SecretKey,
}

#[wasm_bindgen]
impl NorganceEd25519DalekPrivateKey {
    pub fn from_rng(rng: &mut NorganceRng) -> NorganceEd25519DalekPrivateKey {
        let key = ed25519_dalek::SecretKey::generate(&mut rng.rng);
        NorganceEd25519DalekPrivateKey { key }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<NorganceEd25519DalekPrivateKey> {
        let key = match ed25519_dalek::SecretKey::from_bytes(&bytes) {
            Ok(key) => key,
            Err(_) => return Err(NorganceError::InvalidEd25519DalekPrivateKey.into()),
        };

        Ok(NorganceEd25519DalekPrivateKey { key })
    }

    pub fn from_base64(private_key_base64: &str) -> Result<NorganceEd25519DalekPrivateKey> {
        let bytes = match base64::decode(private_key_base64) {
            Ok(bytes) => bytes.into_boxed_slice(),
            Err(_) => return Err(NorganceError::InvalidEd25519DalekPrivateKey.into()),
        };

        NorganceEd25519DalekPrivateKey::from_bytes(&bytes)
    }

    #[must_use]
    pub fn to_base64(&self) -> String {
        base64::encode_config(&self.key.to_bytes().to_vec(), base64::STANDARD_NO_PAD)
    }

    #[must_use]
    pub fn get_public_key(&self) -> NorganceEd25519DalekPublicKey {
        let key = ed25519_dalek::PublicKey::from(&self.key);
        NorganceEd25519DalekPublicKey { key }
    }
}

#[wasm_bindgen]
pub struct NorganceEd25519DalekPublicKey {
    key: ed25519_dalek::PublicKey,
}

#[wasm_bindgen]
impl NorganceEd25519DalekPublicKey {
    pub fn from_base64(public_key_base64: &str) -> Result<NorganceEd25519DalekPublicKey> {
        let bytes = match base64::decode(public_key_base64) {
            Ok(bytes) => bytes.into_boxed_slice(),
            Err(_) => return Err(NorganceError::InvalidEd25519DalekPublicKey.into()),
        };

        let key = match ed25519_dalek::PublicKey::from_bytes(&bytes) {
            Ok(key) => key,
            Err(_) => return Err(NorganceError::InvalidEd25519DalekPublicKey.into()),
        };

        Ok(NorganceEd25519DalekPublicKey { key })
    }

    #[must_use]
    pub fn to_base64(&self) -> String {
        base64::encode_config(self.key.as_bytes().to_vec(), base64::STANDARD_NO_PAD)
    }
}
