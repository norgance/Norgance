mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    //alert("Hello, norgance!");
}

#[wasm_bindgen]
pub fn derivate_citizen_primary_key(name: &str) -> String {
    const ARGON2ID_SETTINGS: argon2::Config = argon2::Config {
        variant: argon2::Variant::Argon2id,
        version: argon2::Version::Version13,
        // These values are low to make it fast enough
        // on slow devices. They are mainly there to make
        // a large scale bruteforce attack a bit more expensive,
        // and also because it's fun.
        // A single blake2b hash could have been good enough.
        mem_cost: 1024, // kb
        time_cost: 2,
        lanes: 1,
        thread_mode: argon2::ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 16,
    };
    
    // The salt is norgance- followed by the first 32 digits of ln(3)
    // (prononced Hélène de Troie in French, for Helen of Troy)
    // https://en.wikipedia.org/wiki/Nothing-up-my-sleeve_number
    const SALT: &[u8] = b"norgance-1.0986122886681096913952452369225";

    /*let input = [name.as_bytes(), &[0x1E], password.as_bytes()].concat();*/

    let hash = argon2::hash_raw(name.as_bytes(), SALT, &ARGON2ID_SETTINGS).unwrap();

    let mut bytes: [u8; 16] = [0; 16];
    bytes.clone_from_slice(&hash[0..16]);

    return uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::Future)
        .build()
        .to_string();
}

#[wasm_bindgen]
pub fn norgance_password_hash(password: &str, size: usize) -> String {
    // The first step is to compute a sha1 of the password
    // because our passwords datasets provides sha1 hashes
    let mut sha1_hasher = sha1::Sha1::new();
    sha1_hasher.update(password.as_bytes());
    let password_sha1 = sha1_hasher.digest().bytes();

    // The next step is about computing a blake2b of the password
    // with some salt specific to norgance. So someone intercepting
    // the request on the network would have to spend quite a lot
    // of ressources to find out what was the origin of the hash

    // The salt is norgance- followed by the first 32 digits of ln(3)
    // (prononced Hélène de Troie in French, for Helen of Troy)
    // https://en.wikipedia.org/wiki/Nothing-up-my-sleeve_number

    let hash = blake2_rfc::blake2b::blake2b(
        size,
        b"norgance-1.0986122886681096913952452369225",
        &password_sha1,
    );
    return hex::encode(hash);
}