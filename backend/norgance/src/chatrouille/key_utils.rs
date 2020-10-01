pub fn gen_private_key() -> x448::Secret {
    let mut rng = rand::thread_rng();
    return x448::Secret::new(&mut rng);
}

pub fn gen_public_key(private_key: &x448::Secret) -> x448::PublicKey {
    return x448::PublicKey::from(private_key);
}

pub fn private_key_to_base64(private_key: &x448::Secret) -> String {
    return base64::encode_config(private_key.as_bytes().to_vec(), base64::STANDARD_NO_PAD);
}

pub fn public_key_to_base64(public_key: &x448::PublicKey) -> String {
    return base64::encode_config(public_key.as_bytes().to_vec(), base64::STANDARD_NO_PAD);
}

pub fn private_key_from_base64(private_key_base64: String) -> Option<x448::Secret> {
    let bytes = match base64::decode(private_key_base64) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };
    match x448::Secret::from_bytes(&bytes) {
        Some(private_key) => return Some(private_key),
        None => return None,
    };
}

pub fn public_key_from_base64(public_key_base64: String) -> Option<x448::PublicKey> {
    let bytes = match base64::decode(public_key_base64) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };
    match x448::PublicKey::from_bytes(&bytes) {
        Some(public_key) => return Some(public_key),
        None => return None,
    };
}

pub fn derive_shared_secret_to_sym_key(
    shared_secret: &x448::SharedSecret,
    key: &[u8],
) -> Result<orion::aead::SecretKey, orion::errors::UnknownCryptoError> {
    const SYMMETRIC_KEY_SIZE: usize = 32;
    let symmetric_key_bytes =
        blake2_rfc::blake2b::blake2b(SYMMETRIC_KEY_SIZE, key, shared_secret.as_bytes());
    let symmetric_key = orion::aead::SecretKey::from_slice(symmetric_key_bytes.as_bytes())?;
    return Ok(symmetric_key);
}
