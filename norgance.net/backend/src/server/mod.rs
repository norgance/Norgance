mod check_password_quality;
mod graphql;
mod handlers;

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, Server};

use crate::db;
use crate::vault;

#[allow(clippy::expect_used)]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn private_key_to_public_key_base64(public_key: &x448::PublicKey) -> String {
    base64::encode_config(public_key.as_bytes(), base64::STANDARD_NO_PAD)
}

fn private_key_sign_base64(public_key: &x448::PublicKey, keypair: &ed25519_dalek::Keypair) -> String {
    use ed25519_dalek::Signer;
    let signature = keypair.sign(public_key.as_bytes());
    base64::encode_config(signature.to_bytes(), base64::STANDARD_NO_PAD)
}

pub struct ServerData {
    db_pool: Arc<db::DbPool>,
    vault_client: Arc<vault::Client>,
    #[cfg(feature = "development")]
    authentication_bearer: Arc<String>,
    private_key_x448: Arc<x448::Secret>,
    public_key_x448_base64: Arc<String>,
    public_key_signature: Arc<String>,
}

impl ServerData {
    pub fn new(
        db_pool: db::DbPool,
        vault_client: vault::Client,
        #[cfg(feature = "development")]
        authentication_bearer: String,
        x448_private_key: x448::Secret,
        ed25519_keypair: ed25519_dalek::Keypair,
    ) -> ServerData {

        let x448_public_key = x448::PublicKey::from(&x448_private_key);
        let public_key_x448_base64 = private_key_to_public_key_base64(&x448_public_key);
        let signature_base64 = private_key_sign_base64(&x448_public_key, &ed25519_keypair);

        ServerData {
            db_pool: Arc::new(db_pool),
            vault_client: Arc::new(vault_client),
            #[cfg(feature = "development")]
            authentication_bearer: Arc::new(authentication_bearer),
            private_key_x448: Arc::new(x448_private_key),
            public_key_x448_base64: Arc::new(public_key_x448_base64),
            public_key_signature: Arc::new(signature_base64),
        }
    }
}

pub async fn server_main(addr: SocketAddr, data: ServerData) {
    let root_node = graphql::new_root_node();
    let data = Arc::new(data);

    let new_service = make_service_fn(move |_| {
        let root_node = Arc::clone(&root_node);
        let data = Arc::clone(&data);

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = Arc::clone(&root_node);
                let data = Arc::clone(&data);

                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::POST, "/chatrouille") => {
                            handlers::chatrouille(
                                req,
                                root_node,
                                Arc::clone(&data.db_pool),
                                Arc::clone(&data.vault_client),
                                Arc::clone(&data.private_key_x448),
                            )
                            .await
                        }
                        (&Method::GET, "/chatrouille_informations") => {
                            handlers::chatrouille_informations(&data.public_key_x448_base64, &data.public_key_signature)
                        }
                        (&Method::GET, "/health") => handlers::health(&data.db_pool),
                        #[cfg(feature = "development")]
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            handlers::graphql(
                                req,
                                root_node,
                                Arc::clone(&data.db_pool),
                                Arc::clone(&data.vault_client),
                                Arc::clone(&data.authentication_bearer),
                            )
                            .await
                        }
                        #[cfg(feature = "development")]
                        (&Method::GET, "/") => juniper_hyper::playground("/graphql", None).await,
                        _ => handlers::not_found(),
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(new_service);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    println!("Listening on http://{}", addr);

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e)
    }
}
