use crate::chatrouille;
use crate::db;
use crate::server::graphql;

extern crate futures;

use hyper::{Body, Request, Response, StatusCode};
use serde_json::json;
use snafu::Snafu;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Snafu)]
pub enum NorganceChatrouilleError {
  #[snafu(display("TooBig"))]
  TooBig,
}

#[allow(clippy::expect_used)]
pub fn json_response(json: &serde_json::value::Value, status: StatusCode) -> Response<Body> {
  Response::builder()
    .status(status)
    .header(hyper::header::CONTENT_TYPE, "application/json")
    .body(Body::from(
      serde_json::to_vec(&json).expect("Unable to serialize json"),
    ))
    .expect("Unable to build json response")
}

pub fn json_ok(json: &serde_json::value::Value) -> Response<Body> {
  json_response(json, StatusCode::OK)
}

pub fn json_error<T>(error: T, status: StatusCode) -> Response<Body>
where
  T: std::fmt::Display,
{
  json_response(
    &json!({
      "error": error.to_string(),
    }),
    status,
  )
}

async fn read_request_body(
  req: Request<Body>,
  limit: usize,
) -> Result<(Vec<u8>, bool), hyper::Error> {
  use futures::TryStreamExt;

  let body = req.into_body();
  body
    .try_fold(
      (Vec::new(), false),
      |(mut data, too_long), chunk| async move {
        if too_long || data.len() + chunk.len() > limit {
          Ok((data, true))
        } else {
          data.extend_from_slice(&chunk);
          Ok((data, false))
        }
      },
    )
    .await
}

type ResultHandler = Result<Response<Body>, hyper::Error>;

pub async fn graphql(
  req: Request<Body>,
  root_node: Arc<graphql::Schema>,
  arc_db_pool: Arc<db::DbPool>,
  authentication_bearer: Cow<'static, str>,
) -> ResultHandler {
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
    citizen_identifier,
  });

  juniper_hyper::graphql(root_node, context_for_query, req).await
}

#[allow(clippy::needless_pass_by_value)]
pub fn health(arc_db_pool: Arc<db::DbPool>) -> ResultHandler {
  let ok = match arc_db_pool.get() {
    Ok(db) => db::health_check(&db).is_ok(),
    Err(_) => false,
  };
  Ok(json_ok(&json!({ "available": ok })))
}

#[allow(clippy::expect_used)]
pub fn not_found() -> ResultHandler {
  Ok(
    Response::builder()
      .status(StatusCode::NOT_FOUND)
      .body(Body::empty())
      .expect("Unable to build not found response"),
  )
}

pub async fn chatrouille(
  req: Request<Body>,
  private_key: Arc<x448::Secret>,
  root_node: Arc<graphql::Schema>,
  arc_db_pool: Arc<db::DbPool>,
) -> ResultHandler {
  use chatrouille::VerifyUnpackedQuerySignature;
  let (body, body_too_long) = read_request_body(req, 4200).await?;

  if body_too_long {
    return Ok(json_response(
      &json!({
        "error": "payload too large"
      }),
      StatusCode::PAYLOAD_TOO_LARGE,
    ));
  }

  let payload = chatrouille::unpack_query(&body, &private_key);

  let unpacked_query = match payload {
    Ok(unpacked_query) => unpacked_query,
    Err(x) => return Ok(json_error(x, StatusCode::UNPROCESSABLE_ENTITY)),
  };

  let graphql_request: juniper::http::GraphQLBatchRequest =
    match serde_json::from_slice(&unpacked_query.payload) {
      Ok(batch) => batch,
      Err(x) => {
        return Ok(json_error(x, StatusCode::BAD_REQUEST));
      }
    };
  let citizen_identifier = Some(String::from("canard"));

  let signature_public_key = ed25519_dalek::PublicKey::from_bytes(&[
    176, 102, 32, 203, 59, 181, 83, 5, 128, 168, 162, 97, 165, 225, 237, 64, 2, 175, 178, 90, 221,
    38, 99, 22, 17, 8, 27, 69, 13, 19, 6, 121,
  ])
  .expect("prout");

  let signed = match unpacked_query.signature {
    Some(signature) => match signature.verify(&signature_public_key) {
      Ok(()) => true,
      Err(x) => {
        return Ok(json_error(x, StatusCode::FORBIDDEN));
      }
    },
    None => false,
  };

  let context_for_query = graphql::Ctx {
    db_pool: arc_db_pool.clone(),
    citizen_identifier,
  };
  let graphql_response = graphql_request
    .execute(&*root_node, &context_for_query)
    .await;

  if !graphql_response.is_ok() {
    return Ok(json_response(
      &json!({
        "error": "bad request"
      }),
      StatusCode::BAD_REQUEST,
    ));
  }

  let response_payload = match serde_json::to_vec(&graphql_response) {
    Ok(response_payload) => response_payload,
    Err(x) => {
      return Ok(json_error(x, StatusCode::INTERNAL_SERVER_ERROR));
    }
  };

  let encrypted_response =
    match chatrouille::pack_response(&response_payload, &unpacked_query.shared_secret) {
      Ok(encrypted_response) => encrypted_response,
      Err(x) => {
        return Ok(json_error(x, StatusCode::INTERNAL_SERVER_ERROR));
      }
    };

  Ok(
    Response::builder()
      .status(StatusCode::OK)
      .header(hyper::header::CONTENT_TYPE, "application/octet-stream")
      .body(Body::from(encrypted_response))
      .expect("Unable to build response"),
  )
}

pub fn chatrouille_public_key() -> ResultHandler {
  Ok(json_ok(&json!({
          "public_key": "abc",
          "signature": "efg"
  })))
}

#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]
#[cfg(test)]
mod tests {
  use super::*;
  use chatrouille::key_utils;
  use tokio_test::block_on;

  fn read_response_body(response: Response<Body>) -> Vec<u8> {
    use futures::TryStreamExt;
    block_on(
      response
        .into_body()
        .try_fold(Vec::new(), |mut data, chunk| async move {
          data.extend_from_slice(&chunk);
          Ok(data)
        }),
    )
    .unwrap()
  }

  fn body_contains(response: Response<Body>, text: &str) -> bool {
    let body = read_response_body(response);
    let body_text = std::str::from_utf8(&body).unwrap();
    body_text.contains(text)
  }

  fn setup_chatrouille() -> (
    Arc<x448::Secret>,
    x448::PublicKey,
    Arc<graphql::Schema>,
    Arc<db::DbPool>,
  ) {
    let private_key = key_utils::gen_private_key();
    let public_key = key_utils::gen_public_key(&private_key);
    let root_node = graphql::new_root_node();
    let db_pool = db::create_connection_pool().expect("Unable to create connection pool");

    (
      Arc::new(private_key),
      public_key,
      root_node,
      Arc::new(db_pool),
    )
  }

  #[test]
  fn test_chatrouille_empty() {
    let (private_key, _, root_node, db_pool) = setup_chatrouille();

    // Empty
    let request = Request::builder().body(Body::empty()).unwrap();
    let response = block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body_contains(response, "too small"));
  }

  #[test]
  fn test_chatrouille_random() {
    use rand::prelude::*;

    let (private_key, _, root_node, db_pool) = setup_chatrouille();

    // Random data
    let mut random_data = [0_u8; 256];
    rand::thread_rng().fill_bytes(&mut random_data);
    // Make sure the prefix is always invalid
    random_data[0] = 0;

    let request = Request::builder()
      .body(Body::from(random_data.to_vec()))
      .unwrap();
    let response = block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body_contains(response, "prefix is invalid"));
  }

  #[test]
  fn test_chatrouille_wrong_public_key() {
    let (private_key, _, root_node, db_pool) = setup_chatrouille();
    let another_private_key = key_utils::gen_private_key();
    let another_public_key = key_utils::gen_public_key(&another_private_key);

    // Valid empty query, but with a wrong public key :-)
    let query = chatrouille::pack_unsigned_query(&[], &another_public_key).unwrap();

    let request = Request::builder().body(Body::from(query.0)).unwrap();

    let response = block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body_contains(response, "Unable to decrypt"));
  }

  #[test]
  fn test_chatrouille_wrong_graphql() {
    let (private_key, public_key, root_node, db_pool) = setup_chatrouille();

    let query = chatrouille::pack_unsigned_query(
      &serde_json::to_vec(&json!({
        "not graphql": true
      }))
      .unwrap(),
      &public_key,
    )
    .unwrap();
    let request = Request::builder().body(Body::from(query.0)).unwrap();
    let response = block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(body_contains(response, "data did not match"));
  }

  #[test]
  fn test_chatrouille_valid_unsigned() {
    let (private_key, public_key, root_node, db_pool) = setup_chatrouille();

    let (query, shared_secret) = chatrouille::pack_unsigned_query(
      &serde_json::to_vec(&json!({
        "operationName": "loadCitizenPublicKey",
        "variables": {
          "identifier": "abcdef"
        },
        "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) {   publicX448 publicX25519Dalek publicEd25519Dalek }}"
      }))
      .unwrap(),
      &public_key,
    )
    .unwrap();
    let request = Request::builder().body(Body::from(query)).unwrap();
    let encrypted_response = block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(encrypted_response.status(), StatusCode::OK);
    let encrypted_body = read_response_body(encrypted_response);
    let response = chatrouille::unpack_response(&encrypted_body, &shared_secret).unwrap();

    //let response_text = std::str::from_utf8(&response).unwrap();
    assert_eq!(response, serde_json::to_vec(&json!({
      "data": { "loadCitizenPublicKeys" : null }
    })).unwrap());
  }
}
