#![allow(clippy::indexing_slicing, clippy::as_conversions)]

pub mod compressor;
pub mod key_utils;

use snafu::{ResultExt, Snafu};

/**
 *  The message format is the following:
 *
 * query: [duck emoji (4 bytes)][mode Query (1 byte)][client x448 public key (56 bytes)][nounce (24 bytes)][data compressed with zlib and encrypted using xchacha20poly1305 (n bytes)][tag (16 bytes)]
 * answer: [duck emoji (4 bytes)][mode Answer (1 byte)][nounce (24 bytes)][data compressed with zlib and encrypted using xchacha20poly1305 (n bytes)][tag (16 bytes)]
 *
 * The duck emoji 🦆 is used as a chatrouille message version, as a warrant canary, and it's cute.
 *
 * Unlike JWE and similar, every algorithm is fixed. If the security of one of the parts of the message requires a change, or if better algorithms exist in the future, it will require a new version with something different than the duck emoji.
 */

#[derive(Snafu, Debug)]
pub enum ChatrouilleError {
  #[snafu(display("Unable to do the diffie Hellman"))]
  DiffieHellmanFail,

  #[snafu(display("Unable to compress the data: {}", source))]
  CompressionError { source: compressor::CompressorError },

  #[snafu(display("Unable to uncompress the data: {}", source))]
  UncompressionError { source: compressor::CompressorError },

  #[snafu(display("Unable to derive the secret to a symmetric key"))]
  KeyDerivationError {
    source: orion::errors::UnknownCryptoError,
  },

  #[snafu(display("Unable to encrypt the data"))]
  EncryptionError {
    source: orion::errors::UnknownCryptoError,
  },

  #[snafu(display("Unable to decrypt the data"))]
  DecryptionError {
    source: orion::errors::UnknownCryptoError,
  },
  #[snafu(display("Unable to load the signature: {}", source))]
  SignatureError {
    source: ed25519_dalek::SignatureError,
  },
  #[snafu(display("Unable to verify the signature: {}", source))]
  VerifySignatureError {
    source: ed25519_dalek::SignatureError,
  },

  #[snafu(display("Invalid mode"))]
  InvalidMode,

  #[snafu(display("Missing keypair"))]
  MissingKeyPair,

  #[snafu(display("Invalid mode in data"))]
  InvalidModeInData,

  #[snafu(display("Data length is too small"))]
  NotEnoughData,

  #[snafu(display("The data prefix is invalid"))]
  InvalidDataPrefix,

  #[snafu(display("Unable to load the encryption key"))]
  KeyLoadingError,
}

pub type Result<T, E = ChatrouilleError> = std::result::Result<T, E>;

#[repr(u8)]
#[derive(Clone, PartialEq)]
pub enum Mode {
  Unknown = 0,
  Query = 81,       // Q
  Response = 82,    // R
  SignedQuery = 83, // S
}

impl From<u8> for Mode {
  fn from(item: u8) -> Self {
    match item {
      81 => Mode::Query,
      82 => Mode::Response,
      83 => Mode::SignedQuery,
      _ => Mode::Unknown,
    }
  }
}

#[allow(clippy::non_ascii_literal)]
const PACKET_VERSION: &[u8] = "🦆".as_bytes();
const PACKET_VERSION_LENGTH: usize = PACKET_VERSION.len();
const MODE_LENGTH: usize = 1;
const CLIENT_PUBLIC_KEY_LENGTH: usize = 56;
const NOUNCE_LENGTH: usize = 24;
const TAG_LENGTH: usize = 16;
const SIGNATURE_LENGTH: usize = 64;
const MINIMUM_QUERY_DATA_LENGTH: usize =
  PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH + NOUNCE_LENGTH + TAG_LENGTH;
const MINIMUM_RESPONSE_DATA_LENGTH: usize =
  PACKET_VERSION_LENGTH + MODE_LENGTH + NOUNCE_LENGTH + TAG_LENGTH;
const MINIMUM_SIGNED_QUERY_DATA_LENGTH: usize = MINIMUM_QUERY_DATA_LENGTH + SIGNATURE_LENGTH;
const SIGNATURE_BLAKE2B_HASH_SIZE: usize = 64;
// Salt for the signature - French revolution - https://en.wikipedia.org/wiki/Nothing-up-my-sleeve_number
const SIGNATURE_BLAKE2B_HASH_SALT: &[u8; 16] = b"chatrouille-1789";

pub struct UnpackedQuery {
  pub payload: Vec<u8>,
  pub mode: Mode,
  pub shared_secret: x448::SharedSecret,
  pub signature: Option<UnpackedQuerySignature>,
}

pub struct UnpackedQuerySignature {
  query_hash: Vec<u8>,
  signature: ed25519_dalek::Signature,
}

pub trait VerifyUnpackedQuerySignature {
  fn verify(&self, public_key: &ed25519_dalek::PublicKey) -> Result<()>;
}

#[allow(dead_code)] // TODO
pub fn pack_signed_query(
  data: &[u8],
  server_public_key: &x448::PublicKey,
  client_keypair: &ed25519_dalek::Keypair,
) -> Result<(Vec<u8>, x448::SharedSecret)> {
  pack_query(data, server_public_key, Some(client_keypair))
}

#[allow(dead_code)] // TODO
pub fn pack_unsigned_query(
  data: &[u8],
  server_public_key: &x448::PublicKey,
) -> Result<(Vec<u8>, x448::SharedSecret)> {
  pack_query(data, server_public_key, None)
}

fn pack_query(
  data: &[u8],
  server_public_key: &x448::PublicKey,
  client_keypair: Option<&ed25519_dalek::Keypair>,
) -> Result<(Vec<u8>, x448::SharedSecret)> {
  let mut rng = rand::thread_rng();
  let client_secret = x448::Secret::new(&mut rng);
  let client_public_key = x448::PublicKey::from(&client_secret);
  let shared_secret = match client_secret.as_diffie_hellman(server_public_key) {
    Some(secret) => secret,
    None => return Err(ChatrouilleError::DiffieHellmanFail),
  };

  let public_key_bytes = client_public_key.as_bytes().to_vec();
  /*assert_eq!(
    public_key_bytes.len(),
    CLIENT_PUBLIC_KEY_LENGTH,
    "The public key length should be {} bytes",
    CLIENT_PUBLIC_KEY_LENGTH
  );*/

  let mode = match client_keypair {
    Some(_) => Mode::SignedQuery,
    None => Mode::Query,
  };

  let encrypted_payload = pack(data, mode, public_key_bytes, &shared_secret, client_keypair)?;

  Ok((encrypted_payload, shared_secret))
}

pub fn pack_response(data: &[u8], shared_secret: &x448::SharedSecret) -> Result<Vec<u8>> {
  pack(data, Mode::Response, vec![], &shared_secret, None)
}

#[allow(clippy::needless_pass_by_value)]
fn pack(
  data: &[u8],
  mode: Mode,
  client_public_key_bytes: Vec<u8>,
  shared_secret: &x448::SharedSecret,
  client_keypair: Option<&ed25519_dalek::Keypair>,
) -> Result<Vec<u8>> {
  let mode_byte = mode.clone() as u8;

  let mut compressed = compressor::compress(&data).context(CompressionError)?;

  // To slightly improve the privacy, we pad all
  // compressed messages with zeros to have a final size which is a multiple of 32.
  let diff_with_32 = compressed.len() % 32;
  if diff_with_32 != 0 {
    compressed.append(&mut vec![0; 32 - diff_with_32]);
  }

  let symmetric_key = key_utils::derive_shared_secret_to_sym_key(shared_secret, &[mode_byte])
    .context(KeyDerivationError)?;

  let mut encrypted = orion::aead::seal(&symmetric_key, &compressed).context(EncryptionError)?;

  let packed_data_length = PACKET_VERSION_LENGTH
    + MODE_LENGTH
    + match mode {
      Mode::Query | Mode::SignedQuery => CLIENT_PUBLIC_KEY_LENGTH,
      Mode::Response => 0,
      Mode::Unknown => return Err(ChatrouilleError::InvalidMode),
    }
    + encrypted.len()
    + match mode {
      Mode::SignedQuery => SIGNATURE_LENGTH,
      Mode::Query | Mode::Response | Mode::Unknown => 0,
    };

  let mut packed_data: Vec<u8> = Vec::with_capacity(packed_data_length);

  packed_data.extend(PACKET_VERSION);
  packed_data.push(mode_byte);
  if mode == Mode::Query || mode == Mode::SignedQuery {
    packed_data.extend(client_public_key_bytes);
  }
  packed_data.append(&mut encrypted);

  if mode == Mode::SignedQuery {
    use blake2_rfc::blake2b::blake2b;
    use ed25519_dalek::Signer;

    let keypair = match client_keypair {
      Some(ckp) => ckp,
      None => return Err(ChatrouilleError::MissingKeyPair),
    };

    // We sign first because appending the data will move the data
    let packet_hash = blake2b(
      SIGNATURE_BLAKE2B_HASH_SIZE,
      SIGNATURE_BLAKE2B_HASH_SALT,
      &packed_data,
    );
    let packet_hash_bytes = packet_hash.as_bytes();
    let signature = keypair.sign(packet_hash_bytes);
    let bytes = &signature.to_bytes();

    packed_data.extend_from_slice(bytes);
  }

  Ok(packed_data)
}

pub fn unpack_query(packed_data: &[u8], private_key: &x448::Secret) -> Result<UnpackedQuery> {
  let data_length = packed_data.len();
  if data_length < MINIMUM_QUERY_DATA_LENGTH {
    return Err(ChatrouilleError::NotEnoughData);
  }

  // If wrong version
  if &packed_data[0..PACKET_VERSION_LENGTH] != PACKET_VERSION {
    return Err(ChatrouilleError::InvalidDataPrefix);
  }

  let mode = Mode::from(packed_data[PACKET_VERSION_LENGTH]);
  if mode == Mode::SignedQuery {
    if data_length < MINIMUM_SIGNED_QUERY_DATA_LENGTH {
      return Err(ChatrouilleError::NotEnoughData);
    }
  } else if mode != Mode::Query {
    return Err(ChatrouilleError::InvalidModeInData);
  }

  let public_key_bytes = &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH
    ..PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH];

  let public_key = match x448::PublicKey::from_bytes(public_key_bytes) {
    Some(public_key) => public_key,
    None => return Err(ChatrouilleError::KeyLoadingError),
  };

  let shared_secret = match private_key.as_diffie_hellman(&public_key) {
    Some(shared_secret) => shared_secret,
    None => return Err(ChatrouilleError::DiffieHellmanFail),
  };

  let mode_byte = mode.clone() as u8;
  let symmetric_key = key_utils::derive_shared_secret_to_sym_key(&shared_secret, &[mode_byte])
    .context(KeyDerivationError)?;

  let aead_bytes = match mode {
    Mode::Query => {
      &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH..data_length]
    }
    Mode::SignedQuery => {
      &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH
        ..data_length - SIGNATURE_LENGTH]
    }
    Mode::Response | Mode::Unknown => return Err(ChatrouilleError::InvalidModeInData),
  };

  let raw_data = unpack(aead_bytes, &symmetric_key)?;

  if mode == Mode::SignedQuery {
    use blake2_rfc::blake2b::blake2b;
    use ed25519_dalek::Signature;
    use std::convert::TryFrom;

    let signature_bytes = &packed_data[data_length - SIGNATURE_LENGTH..data_length];
    let everything_else_bytes = &packed_data[0..data_length - SIGNATURE_LENGTH];
    let packet_hash = blake2b(
      SIGNATURE_BLAKE2B_HASH_SIZE,
      SIGNATURE_BLAKE2B_HASH_SALT,
      &everything_else_bytes,
    );
    let signature = Signature::try_from(signature_bytes).context(SignatureError)?;

    return Ok(UnpackedQuery {
      payload: raw_data,
      mode,
      shared_secret,
      signature: Some(UnpackedQuerySignature {
        query_hash: packet_hash.as_bytes().to_vec(),
        signature,
      }),
    });
  }

  Ok(UnpackedQuery {
    payload: raw_data,
    mode,
    shared_secret,
    signature: None,
  })
}

pub fn unpack_response(packed_data: &[u8], shared_secret: &x448::SharedSecret) -> Result<Vec<u8>> {
  let data_length = packed_data.len();
  if data_length < MINIMUM_RESPONSE_DATA_LENGTH {
    return Err(ChatrouilleError::NotEnoughData);
  }

  // If wrong version
  if &packed_data[0..PACKET_VERSION_LENGTH] != PACKET_VERSION {
    return Err(ChatrouilleError::InvalidDataPrefix);
  }

  let mode = Mode::from(packed_data[PACKET_VERSION_LENGTH]);
  if mode != Mode::Response {
    return Err(ChatrouilleError::InvalidModeInData);
  }

  let mode_byte = mode as u8;
  let symmetric_key = key_utils::derive_shared_secret_to_sym_key(&shared_secret, &[mode_byte])
    .context(KeyDerivationError)?;

  let aead_bytes = &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH..data_length];

  let raw_data = unpack(aead_bytes, &symmetric_key)?;

  Ok(raw_data)
}

fn unpack(encrypted_data: &[u8], symmetric_key: &orion::aead::SecretKey) -> Result<Vec<u8>> {
  let decrypted = orion::aead::open(symmetric_key, encrypted_data).context(DecryptionError)?;

  let raw_data = compressor::decompress(&decrypted).context(UncompressionError)?;
  Ok(raw_data)
}

impl VerifyUnpackedQuerySignature for UnpackedQuerySignature {
  fn verify(&self, public_key: &ed25519_dalek::PublicKey) -> Result<()> {
    public_key
      .verify_strict(&self.query_hash, &self.signature)
      .context(VerifySignatureError)?;
    Ok(())
  }
}

#[allow(clippy::panic, clippy::result_expect_used, clippy::option_expect_used)]
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn main_revival() {
    let bob_secret = x448::Secret::from_bytes(&[
      0x1c, 0x30, 0x6a, 0x7a, 0xc2, 0xa0, 0xe2, 0xe0, 0x99, 0xb, 0x29, 0x44, 0x70, 0xcb, 0xa3,
      0x39, 0xe6, 0x45, 0x37, 0x72, 0xb0, 0x75, 0x81, 0x1d, 0x8f, 0xad, 0xd, 0x1d, 0x69, 0x27,
      0xc1, 0x20, 0xbb, 0x5e, 0xe8, 0x97, 0x2b, 0xd, 0x3e, 0x21, 0x37, 0x4c, 0x9c, 0x92, 0x1b, 0x9,
      0xd1, 0xb0, 0x36, 0x6f, 0x10, 0xb6, 0x51, 0x73, 0x99, 0x2d,
    ])
    .expect("Unwrap bob secret");
    let bob_public_key = x448::PublicKey::from(&bob_secret);

    let keypair = ed25519_dalek::Keypair::from_bytes(&[
      46, 132, 86, 217, 108, 106, 16, 143, 86, 20, 150, 48, 236, 132, 24, 1, 197, 235, 183, 200,
      148, 75, 24, 203, 228, 31, 166, 18, 122, 29, 90, 151, 176, 102, 32, 203, 59, 181, 83, 5, 128,
      168, 162, 97, 165, 225, 237, 64, 2, 175, 178, 90, 221, 38, 99, 22, 17, 8, 27, 69, 13, 19, 6,
      121,
    ])
    .expect("Unwrap keypair");

    let (canard, _canard_secret) =
      pack_signed_query(b"Bonjour le monde.", &bob_public_key, &keypair)
        .expect("pack signed query");

    println!(
      "packed: {}\nlen: {}",
      base64::encode_config(canard.clone(), base64::STANDARD_NO_PAD),
      canard.len(),
    );

    let payload = unpack_query(&canard, &bob_secret);
    match payload {
      Ok(unpacked_query) => {
        println!(
          "payload: {} | mode: {:?}",
          std::str::from_utf8(&unpacked_query.payload).unwrap_or("prout"),
          unpacked_query.mode as u8
        );
        let signed = match unpacked_query.signature {
          Some(signature) => signature.verify(&keypair.public).is_ok(),
          None => false,
        };
        assert_eq!(signed, true);
        let canard_response =
          pack_response(b"Bien le bonjour aussi", &unpacked_query.shared_secret)
            .expect("pack response");
        let payload_response = unpack_response(&canard_response, &unpacked_query.shared_secret);
        match payload_response {
          Ok(payload) => {
            println!(
              "payload response: {}",
              std::str::from_utf8(&payload).unwrap_or("prout"),
            );
          }
          Err(_) => panic!("oops 1"),
        }
      }
      Err(_) => panic!("oops 2"),
    };
  }

  #[test]
  fn test_unsigned() {
    let server_private_key = key_utils::gen_private_key();
    let server_public_key = key_utils::gen_public_key(&server_private_key);

    let query_a = pack_unsigned_query(b"top secret data", &server_public_key).unwrap();
    let query_b = pack_unsigned_query(b"top secret data", &server_public_key).unwrap();

    // Check that each query has a different secret
    assert_ne!(query_a.1.as_bytes().to_vec(), query_b.1.as_bytes().to_vec());

    let unpack_query_a = unpack_query(&query_a.0, &server_private_key).unwrap();
    let unpack_query_b = unpack_query(&query_b.0, &server_private_key).unwrap();

    assert!(unpack_query_a.mode == Mode::Query);
    assert_eq!(unpack_query_a.payload, b"top secret data");
    assert!(unpack_query_a.signature.is_none());

    let shared_secret_a = unpack_query_a.shared_secret;
    let shared_secret_b = unpack_query_b.shared_secret;

    assert_ne!(
      shared_secret_a.as_bytes().to_vec(),
      shared_secret_b.as_bytes().to_vec()
    );

    let response_a = pack_response(b"indeed it's secret", &shared_secret_a).unwrap();
    let response_b = pack_response(b"indeed it's secret", &shared_secret_b).unwrap();
    assert_ne!(response_a, response_b);

    let unpack_response_a = unpack_response(&response_a, &shared_secret_a).unwrap();
    let unpack_response_b = unpack_response(&response_b, &shared_secret_b).unwrap();

    assert_eq!(unpack_response_a, b"indeed it's secret");
    assert_eq!(unpack_response_b, b"indeed it's secret");
  }

  #[test]
  fn test_problems() {
    let server_private_key = key_utils::gen_private_key();
    let server_public_key = key_utils::gen_public_key(&server_private_key);

    // Empty data should work
    pack_unsigned_query(&[], &server_public_key).unwrap();

    // Unpacking empty data should not work
    assert!(unpack_query(&[], &server_private_key).is_err());

    // Building a valid query to modify it later
    let (query, shared_secret) = pack_unsigned_query(b"hei", &server_public_key).unwrap();
    assert!(unpack_query(&query, &server_private_key).is_ok());

    let mut query_with_wrong_version = query.clone();
    query_with_wrong_version[0] = 128;
    assert!(unpack_query(&query_with_wrong_version, &server_private_key).is_err());

    let mut query_with_wrong_mode = query.clone();
    query_with_wrong_mode[PACKET_VERSION_LENGTH] = 128;
    assert!(unpack_query(&query_with_wrong_mode, &server_private_key).is_err());

    // Query that is not supposed to be signed
    query_with_wrong_mode[PACKET_VERSION_LENGTH] = Mode::SignedQuery as u8;
    assert!(unpack_query(&query_with_wrong_mode, &server_private_key).is_err());

    // Wrong aead data - invalid tag
    let mut query_with_weird_aead_data = query.clone();
    query_with_weird_aead_data[query.len()-1] = 128;
    assert!(unpack_query(&query_with_weird_aead_data, &server_private_key).is_err());

    assert!(unpack_response(&[], &shared_secret).is_err());
    
    // Building a valid response to modify it later
    let response = pack_response(b"hei hei", &shared_secret).unwrap();

    let mut response_with_wrong_version = response.clone();
    response_with_wrong_version[0] = 128;
    assert!(unpack_response(&response_with_wrong_version, &shared_secret).is_err());

    let mut response_with_wrong_mode = response.clone();
    response_with_wrong_mode[PACKET_VERSION_LENGTH] = 128;
    assert!(unpack_response(&response_with_wrong_mode, &shared_secret).is_err());
  }
}
