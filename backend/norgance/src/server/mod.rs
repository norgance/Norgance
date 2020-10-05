mod graphql;
mod handlers;

use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, Server};

use crate::db;

#[allow(clippy::result_expect_used)]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
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

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        let arc_db_pool = arc_db_pool.clone();
        let authentication_bearer = authentication_bearer.clone();
        let server_private_key = server_private_key.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let arc_db_pool = arc_db_pool.clone();
                let authentication_bearer = authentication_bearer.clone();
                let server_private_key = server_private_key.clone();

                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            handlers::graphql(req, root_node, arc_db_pool, authentication_bearer)
                                .await
                        }
                        (&Method::POST, "/chatrouille") => {
                            handlers::chatrouille(req, server_private_key).await
                        }
                        (&Method::GET, "/chatrouille_public_key") => {
                            handlers::chatrouille_public_key()
                        }
                        (&Method::GET, "/health") => handlers::health(arc_db_pool),
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
