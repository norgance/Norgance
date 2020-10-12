extern crate base64;
extern crate rand;
extern crate x448;

mod chatrouille;
mod compressor;
mod key_utils;
mod server;
//extern crate branca;
//extern crate orion;

fn main() {

    let chatrouille_mode = std::env::args().nth(1).unwrap_or(String::from("lol"));

    server::server_main();
    let duck = "🦆".as_bytes();
    println!("duck: {:?}", duck);

    let lol1 = base64::decode("eJyNxzENAAAIA0ErxQ0OMADbJ03wP2CB8XLAKi8d+uEAxYwRVQ").unwrap();
    let lol2 = compressor::decompress(lol1).unwrap();
    println!("lol: {}", std::str::from_utf8(&lol2).unwrap());

    let prout = key_utils::gen_private_key();
    println!("private key: {}", key_utils::private_key_to_base64(&prout));
    let payload = "Hello World! Hello World! Hello World! Hello World!";
    let base64_payload = base64::encode(payload);
    println!("uncompressed: {}", base64_payload);

    let compressed_payload = compressor::compress(payload.as_bytes().to_vec()).unwrap();
    println!("first byte: {:?}", compressed_payload[0]);
    let base64_payload = base64::encode_config(compressed_payload, base64::STANDARD_NO_PAD);
    println!("  compressed: {}", base64_payload);

    let alice_secret_base64 =
        b"dImRAHuirOkdmQa1oTbYUGgMpipsuqnxXP/kXeZO2gs3P5FU68QSIjO5BHNXdIdui4GR+TwmJ58=";

    /*let mut rng = rand::thread_rng();
    let alice_secret = x448::Secret::new(&mut rng);
    println!("alice private key: {}", base64::encode(alice_secret.as_bytes().to_vec()));*/
    let alice_secret =
        x448::Secret::from_bytes(&base64::decode(alice_secret_base64.to_vec()).unwrap()).unwrap();

    let alice_public_key = x448::PublicKey::from(&alice_secret);
    let alice_public_key_bytes = alice_public_key.as_bytes().to_vec();
    println!(
        "alice public key length: {}, base64: {}",
        alice_public_key_bytes.len(),
        base64::encode(alice_public_key_bytes),
    );

    let bob_secret = x448::Secret::from_bytes(&[
        0x1c, 0x30, 0x6a, 0x7a, 0xc2, 0xa0, 0xe2, 0xe0, 0x99, 0xb, 0x29, 0x44, 0x70, 0xcb, 0xa3,
        0x39, 0xe6, 0x45, 0x37, 0x72, 0xb0, 0x75, 0x81, 0x1d, 0x8f, 0xad, 0xd, 0x1d, 0x69, 0x27,
        0xc1, 0x20, 0xbb, 0x5e, 0xe8, 0x97, 0x2b, 0xd, 0x3e, 0x21, 0x37, 0x4c, 0x9c, 0x92, 0x1b,
        0x9, 0xd1, 0xb0, 0x36, 0x6f, 0x10, 0xb6, 0x51, 0x73, 0x99, 0x2d,
    ])
    .unwrap();
    let bob_public_key = x448::PublicKey::from(&bob_secret);

    let bob_shared = bob_secret.as_diffie_hellman(&alice_public_key).unwrap();
    let alice_shared = alice_secret.as_diffie_hellman(&bob_public_key).unwrap();

    /*println!(
        "bob shared: {}",
        base64::encode(bob_shared.as_bytes().to_vec())
    );
    println!(
        "alice shared: {}",
        base64::encode(alice_shared.as_bytes().to_vec())
    );

    let symmetric_key = blake2_rfc::blake2b::blake2b(32, &[], alice_shared.as_bytes());
    println!("sym: {}", base64::encode(symmetric_key));

    /*let key = b"supersecretkeyyoushouldnotcommit".to_vec();
    let token = branca::Branca::new(&key).unwrap();
    let ciphertext = token.encode(payload).unwrap();
    println!("{}", ciphertext.as_str()); // "Hello World!"

    let payload2 = token.decode(ciphertext.as_str(), 0).unwrap();
    println!("{}", payload2); // "Hello World!"*/

    let secret_key = orion::aead::SecretKey::from_slice(symmetric_key.as_bytes()).unwrap();
    println!(
        "secret key {}",
        base64::encode(secret_key.unprotected_as_bytes())
    );

    let ciphertext = orion::aead::seal(&secret_key, payload.as_bytes()).unwrap();
    println!("encoded {}", base64::encode(ciphertext.clone()));
    let decrypted_data = orion::aead::open(&secret_key, &ciphertext).unwrap();*/

    /*let canard = chatrouille::pack(
        &alice_public_key,
        &alice_shared,
        chatrouille::Mode::Query,
        payload.as_bytes().to_vec(),
        //vec![]
    )
    .unwrap();*/

    let (canard, canard_secret) =
        chatrouille::pack_query(b"Bonjour le monde".to_vec(), &bob_public_key).unwrap();
    println!(
        "packed: {}\nlen: {}",
        base64::encode_config(canard.clone(), base64::STANDARD_NO_PAD),
        canard.len(),
    );

    let payload = chatrouille::unpack_query(canard, &bob_secret);
    match payload {
        Ok((payload, mode, shared_secret)) => {
            println!(
                "payload: {} | mode: {:?}",
                std::str::from_utf8(&payload).unwrap_or("prout"),
                mode as u8
            );
            let canard_response = chatrouille::pack_response(
                b"Bien le bonjour aussi".to_vec(),
                &shared_secret,
            )
            .unwrap();
            let payload_response = chatrouille::unpack_response(canard_response, &shared_secret);
            match payload_response {
                Ok(payload) => {
                    println!(
                        "payload response: {}",
                        std::str::from_utf8(&payload).unwrap_or("prout"),
                    );
                }
                Err(_) => {
                    println!("oops 1");
                }
            }
        }
        Err(_) => {
            println!("oops");
        }
    };

    //chatrouille::unpack(vec![0, 1, 2], &bob_secret);
    //chatrouille::unpack(vec![0, 1, 2, 3, 4], &bob_secret);
}
