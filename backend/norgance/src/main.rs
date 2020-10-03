#![warn(clippy::all)]

mod chatrouille;
mod db;
mod server;
mod validation;

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
//#[macro_use] extern crate juniper;
#[macro_use]
extern crate lazy_static;

use std::borrow::Cow;
use std::env;
use std::sync::Arc;

embed_migrations!("./migrations");

#[tokio::main]
async fn main() {
    let db_pool = db::create_connection_pool().expect("database");

    {
        let should_run_migrations = env::var("DATABASE_MIGRATIONS")
            .unwrap_or_else(|_| String::from("true"))
            .parse::<lenient_bool::LenientBool>()
            .unwrap_or_default()
            .into();
        if should_run_migrations {
            println!("Running migrations");
            let connection = db_pool.get().expect("pool");
            db::migrate(&connection).expect("migrate");
            println!("OK");
        }
    }

    pretty_env_logger::init();

    //let addr = ([127, 0, 0, 1], 3000).into();
    let addr = ([0, 0, 0, 0], 3000).into();

    // This is obviously only for test
    let bob_secret = x448::Secret::from_bytes(&[
        0x1c, 0x30, 0x6a, 0x7a, 0xc2, 0xa0, 0xe2, 0xe0, 0x99, 0xb, 0x29, 0x44, 0x70, 0xcb, 0xa3,
        0x39, 0xe6, 0x45, 0x37, 0x72, 0xb0, 0x75, 0x81, 0x1d, 0x8f, 0xad, 0xd, 0x1d, 0x69, 0x27,
        0xc1, 0x20, 0xbb, 0x5e, 0xe8, 0x97, 0x2b, 0xd, 0x3e, 0x21, 0x37, 0x4c, 0x9c, 0x92, 0x1b,
        0x9, 0xd1, 0xb0, 0x36, 0x6f, 0x10, 0xb6, 0x51, 0x73, 0x99, 0x2d,
    ])
    .unwrap();
    let bob_public_key = x448::PublicKey::from(&bob_secret);

    let (canard, _canard_secret) =
        chatrouille::pack_unsigned_query(b"Bonjour le monde.".to_vec(), &bob_public_key).unwrap();
    println!(
        "packed: {}\nlen: {}",
        base64::encode_config(canard.clone(), base64::STANDARD_NO_PAD),
        canard.len(),
    );

    let payload = chatrouille::unpack_query(canard, &bob_secret);
    match payload {
        Ok((payload, mode, shared_secret, _signature)) => {
            println!(
                "payload: {} | mode: {:?}",
                std::str::from_utf8(&payload).unwrap_or("prout"),
                mode as u8
            );
            let canard_response =
                chatrouille::pack_response(b"Bien le bonjour aussi".to_vec(), &shared_secret)
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

    let authentication_bearer =
        env::var("AUTHENTICATION_BEARER").unwrap_or_else(|_| String::from("canard"));

    server::server_main(
        addr,
        Arc::new(db_pool),
        Cow::from(authentication_bearer),
        Arc::new(bob_secret),
    )
    .await;
}
