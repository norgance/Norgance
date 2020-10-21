mod graphql;
mod handlers;
mod check_password_quality;

use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, Server};

use crate::db;

#[allow(clippy::expect_used)]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn private_key_to_public_key_base64(private_key: &x448::Secret) -> String {
    let public_key = x448::PublicKey::from(private_key);
    base64::encode_config(public_key.as_bytes(), base64::STANDARD_NO_PAD)
}

pub async fn server_main(
    addr: SocketAddr,
    arc_db_pool: Arc<db::DbPool>,
    authentication_bearer: Cow<'static, str>,
    server_private_key: Arc<x448::Secret>,
) {
    let root_node = graphql::new_root_node();
    let authentication_bearer = authentication_bearer.clone();
    let server_private_key = server_private_key.clone();
    let server_public_key = Arc::new(private_key_to_public_key_base64(&server_private_key));
    //let canard = x448::PublicKey::from(&prout);
    //let server_public_key : Arc<x448::PublicKey> = Arc::new(x448::PublicKey::from(server_private_key));

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        let arc_db_pool = arc_db_pool.clone();
        let authentication_bearer = authentication_bearer.clone();
        let server_private_key = server_private_key.clone();
        let server_public_key = server_public_key.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let arc_db_pool = arc_db_pool.clone();
                #[allow(unused_variables)] // It's used in development mode
                let authentication_bearer = authentication_bearer.clone();
                let server_private_key = server_private_key.clone();
                let server_public_key = server_public_key.clone();

                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::POST, "/chatrouille") => {
                            handlers::chatrouille(req, server_private_key, root_node, arc_db_pool).await
                        }
                        (&Method::GET, "/chatrouille_informations") => {
                            handlers::chatrouille_informations(&server_public_key)
                        }
                        (&Method::GET, "/health") => handlers::health(arc_db_pool),
                        #[cfg(feature = "development")]
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            handlers::graphql(req, root_node, arc_db_pool, authentication_bearer)
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
