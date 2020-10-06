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

#[allow(clippy::result_expect_used)]
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

#[allow(clippy::result_expect_used)]
pub fn not_found() -> ResultHandler {
  Ok(
    Response::builder()
      .status(StatusCode::NOT_FOUND)
      .body(Body::empty())
      .expect("Unable to build not found response"),
  )
}

pub async fn chatrouille(req: Request<Body>, private_key: Arc<x448::Secret>) -> ResultHandler {
  use futures::TryStreamExt;

  let body = req.into_body();
  let (entire_body, body_too_long) = match body
    .try_fold(
      (Vec::new(), false),
      |(mut data, too_long), chunk| async move {
        if too_long || data.len() + chunk.len() > 4200 {
          Ok((data, true))
        } else {
          data.extend_from_slice(&chunk);
          Ok((data, false))
        }
      },
    )
    .await
  {
    Ok(body) => body,
    Err(x) => return Err(x),
  };

  if body_too_long {
    return Ok(json_response(
      &json!({
        "error": "payload too large"
      }),
      StatusCode::PAYLOAD_TOO_LARGE,
    ));
  }

  let lol = entire_body; //base64::decode(entire_body).expect("prout");

  /*let private_key = x448::Secret::from_bytes(&[
    0x1c, 0x30, 0x6a, 0x7a, 0xc2, 0xa0, 0xe2, 0xe0, 0x99, 0xb, 0x29, 0x44, 0x70, 0xcb, 0xa3, 0x39,
    0xe6, 0x45, 0x37, 0x72, 0xb0, 0x75, 0x81, 0x1d, 0x8f, 0xad, 0xd, 0x1d, 0x69, 0x27, 0xc1, 0x20,
    0xbb, 0x5e, 0xe8, 0x97, 0x2b, 0xd, 0x3e, 0x21, 0x37, 0x4c, 0x9c, 0x92, 0x1b, 0x9, 0xd1, 0xb0,
    0x36, 0x6f, 0x10, 0xb6, 0x51, 0x73, 0x99, 0x2d,
  ])
  .unwrap();*/

  let signature_public_key = ed25519_dalek::PublicKey::from_bytes(&[
    176, 102, 32, 203, 59, 181, 83, 5, 128, 168, 162, 97, 165, 225, 237, 64, 2, 175, 178, 90, 221,
    38, 99, 22, 17, 8, 27, 69, 13, 19, 6, 121,
  ])
  .expect("prout");

  let payload = chatrouille::unpack_query(&lol, &private_key);

  match payload {
    Ok(unpacked_query) => {
      use chatrouille::VerifyUnpackedQuerySignature;
      let signed = match unpacked_query.signature {
        Some(signature) => match signature.verify(&signature_public_key) {
          Ok(()) => true,
          Err(x) => {
            return Ok(json_error(x, StatusCode::FORBIDDEN));
          }
        },
        None => false,
      };
      Ok(json_ok(&json!({
        "lol": std::str::from_utf8(&unpacked_query.payload).unwrap_or("prout"),
        "signed": signed
      })))
    }
    Err(x) => Ok(json_error(x, StatusCode::UNPROCESSABLE_ENTITY)),
  }
}

pub fn chatrouille_public_key() -> ResultHandler {
  Ok(json_ok(&json!({
          "public_key": "abc",
          "signature": "efg"
  })))
}
