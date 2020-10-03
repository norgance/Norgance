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
  Query = 81,    // Q
  Response = 82, // R
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

#[allow(dead_code)] // TODO
pub fn pack_signed_query(
  data: Vec<u8>,
  server_public_key: &x448::PublicKey,
  client_keypair: &ed25519_dalek::Keypair,
) -> Result<(Vec<u8>, x448::SharedSecret)>{
  pack_query(data, server_public_key, Some(client_keypair))
}

pub fn pack_unsigned_query(
  data: Vec<u8>,
  server_public_key: &x448::PublicKey,
) -> Result<(Vec<u8>, x448::SharedSecret)>{
  pack_query(data, server_public_key, None)
}

fn pack_query(
  data: Vec<u8>,
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
  assert_eq!(
    public_key_bytes.len(),
    CLIENT_PUBLIC_KEY_LENGTH,
    "The public key length should be {} bytes",
    CLIENT_PUBLIC_KEY_LENGTH
  );

  let mode = match client_keypair {
    Some(_) => Mode::SignedQuery,
    None => Mode::Query,
  };

  let encrypted_payload = pack(data, mode, public_key_bytes, &shared_secret, client_keypair)?;

  Ok((encrypted_payload, shared_secret))
}

pub fn pack_response(data: Vec<u8>, shared_secret: &x448::SharedSecret) -> Result<Vec<u8>> {
  pack(data, Mode::Response, vec![], &shared_secret, None)
}

fn pack(
  data: Vec<u8>,
  mode: Mode,
  client_public_key_bytes: Vec<u8>,
  shared_secret: &x448::SharedSecret,
  client_keypair: Option<&ed25519_dalek::Keypair>,
) -> Result<Vec<u8>> {
  let mode_byte = mode.clone() as u8;

  let mut compressed = compressor::compress(data).context(CompressionError)?;

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
      _ => return Err(ChatrouilleError::InvalidMode),
    }
    + encrypted.len()
    + match mode {
      Mode::SignedQuery => SIGNATURE_LENGTH,
      _ => 0,
    };

  let mut packed_data: Vec<u8> = Vec::with_capacity(packed_data_length);

  packed_data.extend(PACKET_VERSION);
  packed_data.push(mode_byte);
  if mode == Mode::Query {
    packed_data.extend(client_public_key_bytes);
  }
  packed_data.append(&mut encrypted);

  if mode == Mode::SignedQuery {
    if client_keypair.is_none() {
      return Err(ChatrouilleError::MissingKeyPair);
    }
    use ed25519_dalek::Signer;
    // We sign first because appending the data will move the data

    // blake2b
    let signature = client_keypair.unwrap().sign(&packed_data);

    packed_data.extend_from_slice(&signature.to_bytes());
  }

  Ok(packed_data)
}

pub fn unpack_query(
  packed_data: Vec<u8>,
  private_key: &x448::Secret,
) -> Result<(Vec<u8>, Mode, x448::SharedSecret, Option<ed25519_dalek::Signature>)> {
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
    Mode::Query => &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH..data_length],
    Mode::SignedQuery => &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH..data_length - SIGNATURE_LENGTH],
    _ => return Err(ChatrouilleError::InvalidModeInData),
  };

  let raw_data = unpack(aead_bytes, symmetric_key)?;

  if mode == Mode::SignedQuery {
    use std::convert::TryFrom;
    let signature_bytes = &packed_data[data_length-SIGNATURE_LENGTH..data_length];
    let signature = ed25519_dalek::Signature::try_from(signature_bytes).context(SignatureError)?;
    return Ok((raw_data, mode, shared_secret, Some(signature)));
  }

  Ok((raw_data, mode, shared_secret, None))
}

pub fn unpack_response(
  packed_data: Vec<u8>,
  shared_secret: &x448::SharedSecret,
) -> Result<Vec<u8>> {
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

  let raw_data = unpack(aead_bytes, symmetric_key)?;

  Ok(raw_data)
}

fn unpack(encrypted_data: &[u8], symmetric_key: orion::aead::SecretKey) -> Result<Vec<u8>> {
  let decrypted = orion::aead::open(&symmetric_key, encrypted_data).context(DecryptionError)?;

  let raw_data = compressor::decompress(decrypted).context(UncompressionError)?;
  Ok(raw_data)
}
