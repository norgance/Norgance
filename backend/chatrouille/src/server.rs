use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Instant;

#[path = "./proxy.rs"]
mod proxy;

async fn chatrouille_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let start = Instant::now();

  let mut response = Response::new(Body::empty());

  let method = req.method().clone();
  let path = req.uri().path().to_owned();

  match (method.clone(), path.as_str()) {
    (Method::GET, "/") => {
      *response.body_mut() = Body::from("OK");
    }
    (Method::POST, "/chatrouille") => match proxy::proxy().await {
      Ok(body) => *response.body_mut() = Body::from(body),
      Err(error) => match error {
        proxy::ProxyError::NotOK => {
          println!("not ok");
        },
        _ => {
          println!("{}", error.to_string());
        },
      },
    },
    _ => {
      *response.status_mut() = StatusCode::NOT_FOUND;
    }
  };
  let elapsed = start.elapsed();
  println!(
    "{} {} {} - {} ms",
    method,
    path,
    response.status(),
    elapsed.as_millis()
  );
  Ok(response)
}

async fn shutdown_signal() {
  // Wait for the CTRL+C signal
  tokio::signal::ctrl_c()
    .await
    .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
pub async fn server_main() {
  // We'll bind to 127.0.0.1:3000
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

  let make_svc =
    make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(chatrouille_service)) });

  let server = Server::bind(&addr).serve(make_svc);
  let graceful = server.with_graceful_shutdown(shutdown_signal());

  if let Err(e) = graceful.await {
    eprintln!("server error: {}", e);
  }
}
