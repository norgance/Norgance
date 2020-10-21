#![warn(
    clippy::all,
    //clippy::restriction,
    clippy::pedantic,
    clippy::needless_pass_by_value,
    clippy::unwrap_used,
    clippy::clone_on_ref_ptr
)]
#![allow(
    clippy::implicit_return,
    clippy::integer_arithmetic,
    clippy::missing_docs_in_private_items,
    clippy::module_name_repetitions,
    clippy::unreachable,
    clippy::used_underscore_binding,
    clippy::wildcard_imports,
    clippy::else_if_without_else,
    clippy::clone_on_ref_ptr,
    clippy::single_match_else,
    clippy::match_wild_err_arm
)]

mod db;
mod server;
mod validation;
mod vault;

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
#[allow(
    clippy::print_stdout,
    clippy::expect_used,
    clippy::unwrap_used
)]
async fn main() {
    let db_pool = db::create_connection_pool().expect("Unable to create connection pool");

    {
        let should_run_migrations = env::var("DATABASE_MIGRATIONS")
            .unwrap_or_else(|_| String::from("true"))
            .parse::<lenient_bool::LenientBool>()
            .unwrap_or_default()
            .into();
        if should_run_migrations {
            println!("Running migrations");
            let connection = db_pool.get().expect("pool");
            db::migrate(&connection).expect("Unable to migrate database");
            println!("OK");
        }
    }

    pretty_env_logger::init();

    //let addr = ([127, 0, 0, 1], 3000).into();
    let addr = ([0, 0, 0, 0], 3000).into();

    let mut vault_client = vault::Client::from_env().await.expect("Vault client error");
    let server_private_key = vault_client.load_server_private_key().await.expect("Unable to load server private key");

    let authentication_bearer =
        env::var("AUTHENTICATION_BEARER").unwrap_or_else(|_| String::from("development-bearer"));

    server::server_main(
        addr,
        Arc::new(db_pool),
        Cow::from(authentication_bearer),
        Arc::new(server_private_key),
    )
    .await;
}
