fn to_base64_string(data: &[u8]) -> String {
  base64::encode_config(data, base64::STANDARD_NO_PAD)
}

pub fn new_keys() {
  use chatrouille::key_utils;
  println!("Generating random new keys");

  let x448_private_key = key_utils::gen_private_key();
  let x448_public_key = key_utils::gen_public_key(&x448_private_key);

  println!("x448_private_key: {}", to_base64_string(x448_private_key.as_bytes()));
  println!("x448_public_key: {}", to_base64_string(x448_public_key.as_bytes()));

  let ed25519_keypair = key_utils::gen_ed25519_keypair();
  println!("ed25519_private_key: {}", to_base64_string(ed25519_keypair.secret.as_bytes()));
  println!("ed25519_public_key: {}", to_base64_string(ed25519_keypair.public.as_bytes()));

  let x25519_secret = key_utils::gen_x25519_static_secret();
  let x25519_public_key = x25519_dalek::PublicKey::from(&x25519_secret);
  println!("x25519_private_key: {}", to_base64_string(&x25519_secret.to_bytes()));
  println!("x25519_public_key: {}", to_base64_string(x25519_public_key.as_bytes()));
  println!("----");
}