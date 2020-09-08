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
pub fn derivate_citizen_primary_key(name: &str, password: &str) -> String {
    const ARGON2ID_SETTINGS: argon2::Config = argon2::Config {
        variant: argon2::Variant::Argon2id,
        version: argon2::Version::Version13,
        mem_cost: 4096,
        //time_cost: 10,
        time_cost: 3,
        lanes: 1,
        thread_mode: argon2::ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 16,
    };
    const SALT: &[u8] = b"vive le roi des canards";

    let input = [name.as_bytes(), &[0x1E], password.as_bytes()].concat();

    let hash = argon2::hash_raw(&input, SALT, &ARGON2ID_SETTINGS).unwrap();

    let mut bytes: [u8; 16] = [0; 16];
    bytes.clone_from_slice(&hash[0..16]);

    return uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::Future)
        .build()
        .to_string();
}
