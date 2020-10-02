pub mod graphql;

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use hyper::{Method, StatusCode};
use serde_json::json;

use crate::db;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn json_response(json: &serde_json::value::Value) -> Response<hyper::body::Body> {
    return Response::builder()
        .status(200)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&json).unwrap()))
        .unwrap();
}

pub async fn server_main(
    arc_db_pool: Arc<db::DbPool>,
    addr: SocketAddr,
    authentication_bearer: std::borrow::Cow<'static, str>,
) {

    let root_node = graphql::new_root_node();
    let authentication_bearer = authentication_bearer.clone();

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        let arc_db_pool = arc_db_pool.clone();
        let authentication_bearer = authentication_bearer.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let arc_db_pool = arc_db_pool.clone();
                let authentication_bearer = authentication_bearer.clone();

                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => juniper_hyper::playground("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            let headers = req.headers();

                            if !match headers.get("authentication") {
                                Some(h) => match h.to_str() {
                                    Ok(h) => h == authentication_bearer,
                                    Err(_) => false,
                                },
                                None => false,
                            } {
                                let mut response = Response::new(Body::empty());
                                *response.status_mut() = StatusCode::FORBIDDEN;
                                return Ok(response);
                            }

                            let citizen_identifier = match headers.get("citizen") {
                                Some(h) => match h.to_str() {
                                    Ok(h) => Some(String::from(h)),
                                    Err(_) => None,
                                },
                                None => None,
                            };

                            let context_for_query = Arc::new(graphql::Ctx {
                                db_pool: arc_db_pool.clone(),
                                citizen_identifier: citizen_identifier,
                            });

                            juniper_hyper::graphql(root_node, context_for_query, req).await
                        }
                        (&Method::GET, "/health") => {
                            let ok = match arc_db_pool.get() {
                                Ok(db) => match db::health_check(&db) {
                                    Ok(_) => true,
                                    Err(_) => false,
                                },
                                Err(_) => false,
                            };
                            Ok(json_response(&json!({ "available": ok })))
                        }
                        (&Method::POST, "/chatrouille") => {
                            let response = Response::new(Body::empty());
                            Ok(response)
                        }
                        (&Method::GET, "/chatrouille_public_key") => Ok(json_response(&json!({
                                "public_key": "abc",
                                "signature": "efg"
                        }))),
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            Ok(response)
                        }
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
