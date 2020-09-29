mod db;
mod graphql;
mod models;
mod schema;

use std::sync::Arc;
use std::env;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Response, Server, StatusCode,
};

#[macro_use]
extern crate juniper;

#[macro_use]
extern crate diesel_migrations;
embed_migrations!("./migrations");

#[macro_use]
extern crate diesel;

#[tokio::main]
async fn main() {

    let db_pool = db::create_connection_pool().expect("database");

    {
        let should_run_migrations = env::var("DATABASE_MIGRATIONS")
            .unwrap_or(String::from("true"))
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

    let arc_db_pool = Arc::new(db_pool);

    let root_node = graphql::new_root_node();

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        //let ctx = db.clone();
        let arc_db_pool = arc_db_pool.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                //let ctx = ctx.clone();
                let arc_db_pool = arc_db_pool.clone();
                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => juniper_hyper::playground("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            let citizen_identifier = match req.headers().get("citizen") {
                                Some(h) => match h.to_str() {
                                    Ok(h) => Some(String::from(h)),
                                    Err(_) => None,
                                },
                                None => None,
                            };

                            /*let arc_db_pool = arc_db_pool.clone();
                            let db_connection = match (&arc_db_pool).get() {
                                Ok(c) => c,
                                Err(error) => {
                                    eprintln!("Unable to get db connection: {}", error);
                                    let mut response = Response::new(Body::empty());
                                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                    return Ok(response);
                                }
                            };*/

                            /*(&arc_db_pool).commit();

                            db::migrate(&db_connection).expect("migrate");*/
                            
                            let prout = Arc::new(graphql::Ctx {
                                dbPool: arc_db_pool.clone(),
                                citizenIdentifier: citizen_identifier,//: Some(String::from("prout")),
                            });

                            //ctx.citizen_identifier = Some(String::from("canard"));
                            juniper_hyper::graphql(root_node, prout, req).await
                        }
                        (&Method::POST, "/lapin") => {
                            let mut response = Response::new(Body::empty());
                            *response.body_mut() = Body::from("{\"available\":true}");
                            Ok(response)
                        }
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
    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
}
