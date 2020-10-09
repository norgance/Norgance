#[allow(dead_code)]
pub fn gen_private_key() -> x448::Secret {
    let mut rng = rand::thread_rng();
    x448::Secret::new(&mut rng)
}

#[allow(dead_code)]
pub fn gen_public_key(private_key: &x448::Secret) -> x448::PublicKey {
    x448::PublicKey::from(private_key)
}

#[allow(dead_code)]
pub fn private_key_to_base64(private_key: &x448::Secret) -> String {
    base64::encode_config(private_key.as_bytes().to_vec(), base64::STANDARD_NO_PAD)
}

#[allow(dead_code)]
pub fn public_key_to_base64(public_key: &x448::PublicKey) -> String {
    base64::encode_config(public_key.as_bytes().to_vec(), base64::STANDARD_NO_PAD)
}

#[allow(dead_code)]
pub fn private_key_from_base64(private_key_base64: &str) -> Option<x448::Secret> {
    let bytes = match base64::decode(private_key_base64) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };
    x448::Secret::from_bytes(&bytes)
}

#[allow(dead_code)]
pub fn public_key_from_base64(public_key_base64: &str) -> Option<x448::PublicKey> {
    let bytes = match base64::decode(public_key_base64) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };
    x448::PublicKey::from_bytes(&bytes)
}

pub fn derive_shared_secret_to_sym_key(
    shared_secret: &x448::SharedSecret,
    key: &[u8],
) -> Result<orion::aead::SecretKey, orion::errors::UnknownCryptoError> {
    const SYMMETRIC_KEY_SIZE: usize = 32;
    let symmetric_key_bytes =
        blake2_rfc::blake2b::blake2b(SYMMETRIC_KEY_SIZE, key, shared_secret.as_bytes());
    let symmetric_key = orion::aead::SecretKey::from_slice(symmetric_key_bytes.as_bytes())?;
    Ok(symmetric_key)
}

#[allow(clippy::panic,clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_key() {
        let private_key = gen_private_key();

        // Check that they are at least somehow random
        for _ in 1..129 {
            assert_ne!(
                private_key.as_bytes().to_vec(),
                gen_private_key().as_bytes().to_vec()
            );
        }

        let pk_to_base64 = private_key_to_base64(&private_key);
        let pk_from_base64 = private_key_from_base64(&pk_to_base64).unwrap();
        assert_eq!(
            private_key.as_bytes().to_vec(),
            pk_from_base64.as_bytes().to_vec()
        );
    }

    #[test]
    fn test_public_key() {
        let private_key = gen_private_key();
        let public_key = gen_public_key(&private_key);
        let pk_to_base64 = public_key_to_base64(&public_key);
        let pk_from_base64 = public_key_from_base64(&pk_to_base64).unwrap();
        assert_eq!(
            public_key.as_bytes().to_vec(),
            pk_from_base64.as_bytes().to_vec()
        );
    }

    #[test]
    fn test_shared_secrets_sym_key() {
        let bob_secret = x448::Secret::from_bytes(&[
            236, 70, 161, 17, 6, 196, 129, 192, 255, 113, 126, 233, 118, 153, 1, 136, 228, 186,
            113, 197, 126, 162, 242, 84, 183, 235, 252, 25, 201, 195, 144, 225, 20, 9, 51, 13, 249,
            44, 26, 123, 91, 224, 74, 87, 125, 223, 70, 133, 194, 192, 86, 74, 56, 182, 246, 168,
        ])
        .unwrap();
        let alice_secret = x448::Secret::from_bytes(&[
            100, 54, 129, 24, 66, 164, 48, 36, 193, 31, 194, 90, 223, 40, 169, 60, 117, 96, 77,
            126, 106, 92, 153, 73, 185, 184, 31, 6, 24, 81, 128, 247, 121, 249, 53, 99, 12, 213,
            152, 234, 7, 47, 132, 253, 53, 234, 212, 173, 253, 62, 176, 171, 245, 243, 79, 225,
        ])
        .unwrap();

        let bob_public = gen_public_key(&bob_secret);
        let alice_public = gen_public_key(&alice_secret);

        let shared_secret_ba = bob_secret.as_diffie_hellman(&alice_public).unwrap();
        let shared_secret_ab = alice_secret.as_diffie_hellman(&bob_public).unwrap();

        // Test the lib a little bit
        assert_eq!(
            shared_secret_ab.as_bytes().to_vec(),
            shared_secret_ba.as_bytes().to_vec()
        );

        let sym_key = derive_shared_secret_to_sym_key(&shared_secret_ab, &[1, 2, 3]).unwrap();
        assert_eq!(
            sym_key.unprotected_as_bytes(),
            &[
                27, 195, 70, 25, 82, 218, 193, 2, 155, 254, 42, 162, 92, 143, 237, 71, 174, 228,
                116, 178, 116, 107, 202, 87, 5, 119, 7, 193, 145, 129, 221, 196
            ]
        );
        let sym_key_2 = derive_shared_secret_to_sym_key(&shared_secret_ab, &[4]).unwrap();
        assert_eq!(
            sym_key_2.unprotected_as_bytes(),
            &[
                67, 29, 127, 187, 249, 2, 175, 181, 54, 250, 230, 64, 170, 148, 206, 242, 112, 19,
                72, 250, 49, 89, 144, 20, 177, 75, 144, 178, 65, 73, 110, 37
            ]
        );
    }

    #[test]
    fn test_broken_base64() {
        assert!(private_key_from_base64("totally not base64").is_none());
        assert!(public_key_from_base64("totally not base64").is_none());
    }
}
