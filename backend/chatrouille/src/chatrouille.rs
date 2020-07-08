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

#[path = "./compressor.rs"]
mod compressor;
#[path = "./key_utils.rs"]
mod key_utils;

#[repr(u8)]
#[derive(Clone, PartialEq)]
pub enum Mode {
  Unknown = 0,
  Query = 81,    // Q
  Response = 82, // R
}

impl From<u8> for Mode {
  fn from(item: u8) -> Self {
    match item {
      81 => Mode::Query,
      82 => Mode::Response,
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
const MINIMUM_QUERY_DATA_LENGTH: usize =
  PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH + NOUNCE_LENGTH + TAG_LENGTH;
const MINIMUM_RESPONSE_DATA_LENGTH: usize =
  PACKET_VERSION_LENGTH + MODE_LENGTH + NOUNCE_LENGTH + TAG_LENGTH;

pub fn pack_query(
  data: Vec<u8>,
  server_public_key: &x448::PublicKey,
) -> Option<(Vec<u8>, x448::SharedSecret)> {
  let mut rng = rand::thread_rng();
  let client_secret = x448::Secret::new(&mut rng);
  let client_public_key = x448::PublicKey::from(&client_secret);
  let shared_secret = match client_secret.as_diffie_hellman(server_public_key) {
    Some(secret) => secret,
    None => return None,
  };

  let public_key_bytes = client_public_key.as_bytes().to_vec();
  assert_eq!(
    public_key_bytes.len(),
    CLIENT_PUBLIC_KEY_LENGTH,
    "The public key length should be {} bytes",
    CLIENT_PUBLIC_KEY_LENGTH
  );

  let encrypted_payload = match pack(data, Mode::Query, public_key_bytes, &shared_secret) {
    Some(payload) => payload,
    None => return None,
  };

  return Some((encrypted_payload, shared_secret));
}

pub fn pack_response(data: Vec<u8>, shared_secret: &x448::SharedSecret) -> Option<Vec<u8>> {
  return pack(data, Mode::Response, vec![], &shared_secret);
}

fn pack(
  data: Vec<u8>,
  mode: Mode,
  client_public_key_bytes: Vec<u8>,
  shared_secret: &x448::SharedSecret,
) -> Option<Vec<u8>> {
  let mode_byte = mode.clone() as u8;

  let compressed = match compressor::compress(data) {
    Some(compressed) => compressed,
    None => return None,
  };

  let symmetric_key = match key_utils::derive_shared_secret_to_sym_key(shared_secret, &[mode_byte])
  {
    Ok(symmetric_key) => symmetric_key,
    Err(_) => return None,
  };

  let mut encrypted = match orion::aead::seal(&symmetric_key, &compressed) {
    Ok(encrypted) => encrypted,
    Err(_) => return None,
  };

  let packed_data_length = PACKET_VERSION_LENGTH
    + MODE_LENGTH
    + match mode {
      Mode::Query => CLIENT_PUBLIC_KEY_LENGTH,
      Mode::Response => 0,
      _ => panic!("no valid mode"),
    }
    + encrypted.len();

  let mut packed_data: Vec<u8> = Vec::with_capacity(packed_data_length);

  //println!("capacity: {}", packed_data.capacity());
  packed_data.extend(PACKET_VERSION);
  packed_data.push(mode_byte);
  if mode == Mode::Query {
    packed_data.extend(client_public_key_bytes);
  }
  packed_data.append(&mut encrypted);
  //println!("len: {}", packed_data.len());
  return Some(packed_data);
}

pub fn unpack_query(
  packed_data: Vec<u8>,
  private_key: &x448::Secret,
) -> Option<(Vec<u8>, Mode, x448::SharedSecret)> {
  let data_length = packed_data.len();
  if data_length < MINIMUM_QUERY_DATA_LENGTH {
    return None;
  }

  // If wrong version
  if &packed_data[0..PACKET_VERSION_LENGTH] != PACKET_VERSION {
    return None;
  }

  let mode = Mode::from(packed_data[PACKET_VERSION_LENGTH]);
  if mode != Mode::Query {
    return None;
  }

  let public_key_bytes = &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH
    ..PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH];
  println!("public key bytes U: {:?}", public_key_bytes);

  let public_key = match x448::PublicKey::from_bytes(public_key_bytes) {
    Some(public_key) => public_key,
    None => return None,
  };

  let shared_secret = match private_key.as_diffie_hellman(&public_key) {
    Some(shared_secret) => shared_secret,
    None => return None,
  };

  let mode_byte = mode.clone() as u8;
  let symmetric_key = match key_utils::derive_shared_secret_to_sym_key(&shared_secret, &[mode_byte])
  {
    Ok(symmetric_key) => symmetric_key,
    Err(_) => return None,
  };

  let aead_bytes =
    &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH + CLIENT_PUBLIC_KEY_LENGTH..data_length];
  //println!("symmetric key: {:?}", symmetric_key.as_bytes());

  let uncompressed = match unpack(aead_bytes, symmetric_key) {
    Some(data) => data,
    None => return None,
  };

  return Some((uncompressed, mode, shared_secret));
}

pub fn unpack_response(packed_data: Vec<u8>, shared_secret: &x448::SharedSecret) -> Option<Vec<u8>> {
  let data_length = packed_data.len();
  if data_length < MINIMUM_RESPONSE_DATA_LENGTH {
    return None;
  }

  // If wrong version
  if &packed_data[0..PACKET_VERSION_LENGTH] != PACKET_VERSION {
    return None;
  }

  let mode = Mode::from(packed_data[PACKET_VERSION_LENGTH]);
  if mode != Mode::Response {
    return None;
  }

  let mode_byte = mode.clone() as u8;
  let symmetric_key = match key_utils::derive_shared_secret_to_sym_key(&shared_secret, &[mode_byte])
  {
    Ok(symmetric_key) => symmetric_key,
    Err(_) => return None,
  };

  let aead_bytes =
    &packed_data[PACKET_VERSION_LENGTH + MODE_LENGTH ..data_length];

  let uncompressed = match unpack(aead_bytes, symmetric_key) {
    Some(data) => data,
    None => return None,
  };

  return Some(uncompressed);
}

fn unpack(encrypted_data: &[u8], symmetric_key: orion::aead::SecretKey) -> Option<Vec<u8>> {
  let decrypted = match orion::aead::open(&symmetric_key, encrypted_data) {
    Ok(encrypted) => encrypted,
    Err(_) => return None,
  };
  return match compressor::decompress(decrypted) {
    Some(compressed) => Some(compressed),
    None => None,
  };
}
