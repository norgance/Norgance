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

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(bound = "juniper::InputValue<S>: serde::Deserialize<'de>")] // ??
pub struct NorganceChatrouilleContainer<S = juniper::DefaultScalarValue>
where
  S: juniper::ScalarValue,
{
  graphql: juniper::http::GraphQLBatchRequest<S>,

  #[serde(rename = "citizenIdentifier")]
  //#[serde(bound(deserialize = "juniper::InputValue<S>: serde::Deserialize<'de> + serde::Serialize"))]
  citizen_identifier: Option<String>,
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

#[allow(clippy::too_many_lines)]
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

  let graphql_request: NorganceChatrouilleContainer =
    match serde_json::from_slice(&unpacked_query.payload) {
      Ok(batch) => batch,
      Err(x) => {
        return Ok(json_error(x, StatusCode::BAD_REQUEST));
      }
    };
  let citizen_identifier = graphql_request.citizen_identifier;

  if citizen_identifier.is_some() {
    let signature = match unpacked_query.signature {
      None => {
        return Ok(json_response(
          &json!({
            "error": "citizenIdentifier requires a signed query"
          }),
          StatusCode::FORBIDDEN,
        ));
      }
      Some(signature) => signature,
    };

    let db_connection = match arc_db_pool.get() {
      Ok(db_connection) => db_connection,
      Err(_) => {
        return Ok(json_response(
          &json!({
            "error": "Unable to query the database"
          }),
          StatusCode::INTERNAL_SERVER_ERROR,
        ));
      }
    };

    #[allow(clippy::unwrap_used)]
    let public_key = match db::load_citizen_public_ed25519_dalek(
      &db_connection,
      &citizen_identifier.clone().unwrap(),
    ) {
      Ok(public_key) => match public_key {
        Some(public_key) => public_key,
        None => {
          return Ok(json_response(
            &json!({
              "error": "Unauthorized citizen identifier"
            }),
            StatusCode::FORBIDDEN,
          ));
        }
      },
      Err(x) => {
        return Ok(json_error(x, StatusCode::INTERNAL_SERVER_ERROR));
      }
    };

    if signature.verify(&public_key).is_err() {
      return Ok(json_response(
        &json!({
          "error": "Unauthorized citizen identifier"
        }),
        StatusCode::FORBIDDEN,
      ));
    };
  }

  let context_for_query = graphql::Ctx {
    db_pool: arc_db_pool.clone(),
    citizen_identifier,
  };
  let graphql_response = graphql_request
    .graphql
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

  fn random_string(size: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;
    let mut rng = thread_rng();
    iter::repeat(())
      .map(|()| rng.sample(Alphanumeric))
      .take(size)
      .collect()
  }

  fn create_test_citizen_in_db(db: &db::DbPooledConnection) -> (String, ed25519_dalek::Keypair) {
    use crate::db::models::{Citizen, NewCitizen};

    let identifier = random_string(64);
    let access_key = random_string(64);
    let keypair_ed25519 = key_utils::gen_ed25519_keypair();
    let private_x25519 = key_utils::gen_x25519_static_secret();
    let public_x25519 = x25519_dalek::PublicKey::from(&private_x25519);
    let private_x448 = key_utils::gen_private_key();
    let public_x448 = key_utils::gen_public_key(&private_x448);

    let private_secret_key = orion::aead::SecretKey::generate(32).unwrap();
    let aead_data = orion::aead::seal(&private_secret_key, b"secret").unwrap();

    let new_citizen = NewCitizen {
      identifier: &identifier,
      access_key: &access_key,
      public_x448: &base64::encode_config(public_x448.as_bytes(), base64::STANDARD_NO_PAD),
      public_x25519_dalek: &base64::encode_config(
        public_x25519.as_bytes(),
        base64::STANDARD_NO_PAD,
      ),
      public_ed25519_dalek: &base64::encode_config(
        keypair_ed25519.public.as_bytes(),
        base64::STANDARD_NO_PAD,
      ),
      aead_data: &base64::encode_config(aead_data, base64::STANDARD_NO_PAD),
    };

    {
      use crate::db::schema::citizens;
      use diesel::prelude::*;
      let _citizen: Citizen = diesel::insert_into(citizens::table)
        .values(&new_citizen)
        .get_result(db)
        .unwrap();
    }

    (identifier, keypair_ed25519)
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
    assert!(body_contains(response, "missing field"));
  }

  #[test]
  fn test_chatrouille_valid_unsigned() {
    let (private_key, public_key, root_node, db_pool) = setup_chatrouille();

    let (query, shared_secret) = chatrouille::pack_unsigned_query(
      &serde_json::to_vec(&json!({
        "graphql": {
          "operationName": "loadCitizenPublicKey",
          "variables": {
            "identifier": "abcdef"
          },
          "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicX448 publicX25519Dalek publicEd25519Dalek }}"
        }
      }))
      .unwrap(),
      &public_key,
    )
    .unwrap();
    let request = Request::builder().body(Body::from(query)).unwrap();
    let encrypted_response =
      block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(encrypted_response.status(), StatusCode::OK);
    let encrypted_body = read_response_body(encrypted_response);
    let response = chatrouille::unpack_response(&encrypted_body, &shared_secret).unwrap();

    //let response_text = std::str::from_utf8(&response).unwrap();
    assert_eq!(
      response,
      serde_json::to_vec(&json!({
        "data": { "loadCitizenPublicKeys" : null }
      }))
      .unwrap()
    );
  }
  #[test]
  fn test_chatrouille_unvalid_unsigned() {
    let (private_key, public_key, root_node, db_pool) = setup_chatrouille();

    let (query, _) = chatrouille::pack_unsigned_query(
      &serde_json::to_vec(&json!({
        "graphql": {
          "operationName": "loadCitizenPublicKey",
          "variables": {
            "identifier": "abcdef"
          },
          "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicX448 publicX25519Dalek publicEd25519Dalek }}"
        },
        "citizenIdentifier": "canard"
      }))
      .unwrap(),
      &public_key,
    )
    .unwrap();

    let request = Request::builder().body(Body::from(query)).unwrap();
    let response = block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(body_contains(response, "signed"));
  }
  #[test]
  fn test_chatrouille_valid_signed() {
    let (private_key, public_key, root_node, db_pool) = setup_chatrouille();

    let db = db_pool.get().expect("Database connection failed");
    let (identifier, keypair) = create_test_citizen_in_db(&db);

    let (query, shared_secret) = chatrouille::pack_signed_query(
      &serde_json::to_vec(&json!({
        "graphql": {
          "operationName": "loadCitizenPublicKey",
          "variables": {
            "identifier": identifier
          },
          "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicEd25519Dalek }}"
        },
        "citizenIdentifier": identifier
      }))
      .unwrap(),
      &public_key,
      &keypair,
    )
    .unwrap();
    let request = Request::builder().body(Body::from(query)).unwrap();
    let encrypted_response =
      block_on(chatrouille(request, private_key, root_node, db_pool)).unwrap();
    assert_eq!(encrypted_response.status(), StatusCode::OK);
    let encrypted_body = read_response_body(encrypted_response);
    let response = chatrouille::unpack_response(&encrypted_body, &shared_secret).unwrap();

    let response_text = std::str::from_utf8(&response).unwrap();
    let ed25519_dalek_base64 = &base64::encode_config(
      keypair.public.as_bytes(),
      base64::STANDARD_NO_PAD,
    );

    assert_eq!(
     response_text,
      serde_json::to_string(&json!({
        "data": {
          "loadCitizenPublicKeys" : {
            "publicEd25519Dalek": ed25519_dalek_base64
          }
        }
      }))
      .unwrap()
    );
  }
}
