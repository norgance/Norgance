use crate::db;
use crate::server::graphql;
use crate::vault;

extern crate futures;

use hyper::{Body, Request, Response, StatusCode};
use serde_json::json;
use snafu::Snafu;
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

    #[serde(rename = "exp")]
    expiration_time: u64,
}

#[cfg(feature = "development")]
const ACCESS_CONTROL_ORIGIN: &str = "*";
#[cfg(not(feature = "development"))]
const ACCESS_CONTROL_ORIGIN: &str = "https://norgance.net";

#[allow(clippy::expect_used)]
pub fn json_response(json: &serde_json::value::Value, status: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .header("Access-Control-Allow-Origin", ACCESS_CONTROL_ORIGIN)
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
    body.try_fold(
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

fn get_timestamp() -> Result<u64, Response<Body>> {
    let server_time =
        match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(t) => t,
            Err(_) => {
                return Err(json_response(
                    &json!({
                      "error": "Unable to get server time"
                    }),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        };
    let server_timestamp = server_time.as_secs();
    Ok(server_timestamp)
}

type ResultHandler = Result<Response<Body>, hyper::Error>;

#[allow(dead_code)]
pub async fn graphql(
    req: Request<Body>,
    root_node: Arc<graphql::Schema>,
    arc_db_pool: Arc<db::DbPool>,
    vault_client: Arc<vault::Client>,
    authentication_bearer: Arc<String>,
) -> ResultHandler {
    let headers = req.headers();

    if !match headers.get("authentication") {
        Some(h) => match h.to_str() {
            Ok(h) => h == authentication_bearer.as_str(),
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
        db_pool: arc_db_pool,
        vault_client,
        citizen_identifier,
    });

    juniper_hyper::graphql(root_node, context_for_query, req).await
}

#[allow(clippy::needless_pass_by_value)]
pub fn health(db_pool: &db::DbPool) -> ResultHandler {
    let ok = match db_pool.get() {
        Ok(db) => db::health_check(&db).is_ok(),
        Err(_) => false,
    };
    Ok(json_ok(&json!({ "available": ok })))
}

#[allow(clippy::expect_used)]
pub fn not_found() -> ResultHandler {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .expect("Unable to build not found response"))
}

#[allow(clippy::too_many_lines)]
pub async fn chatrouille(
    req: Request<Body>,
    root_node: Arc<graphql::Schema>,
    arc_db_pool: Arc<db::DbPool>,
    vault_client: Arc<vault::Client>,
    private_key: Arc<x448::Secret>,
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

    let server_timestamp = match get_timestamp() {
        Ok(t) => t,
        Err(response) => return Ok(response),
    };
    if graphql_request.expiration_time < server_timestamp {
        return Ok(json_response(
            &json!({
              "error": "The request has expired"
            }),
            StatusCode::GONE,
        ));
    }

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
        let public_key =
            match db::load_citizen_access_key(&db_connection, &citizen_identifier.clone().unwrap())
            {
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
        db_pool: Arc::clone(&arc_db_pool),
        citizen_identifier,
        vault_client,
    };
    let graphql_response = graphql_request
        .graphql
        .execute(&*root_node, &context_for_query)
        .await;
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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(hyper::header::CONTENT_TYPE, "application/octet-stream")
        .header("Access-Control-Allow-Origin", ACCESS_CONTROL_ORIGIN)
        .body(Body::from(encrypted_response))
        .expect("Unable to build response"))
}

pub fn chatrouille_information(public_key: &str, signature: &str) -> ResultHandler {
    let time = match get_timestamp() {
        Ok(t) => t,
        Err(response) => return Ok(response),
    };

    Ok(json_ok(&json!({
            "public_key_x448": public_key,
            "public_key_x448_signature": signature,
            "time": time,
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
        Arc<vault::Client>,
    ) {
        let private_key = key_utils::gen_private_key();
        let public_key = key_utils::gen_public_key(&private_key);
        let root_node = graphql::new_root_node();
        let db_pool = db::create_connection_pool().expect("Unable to create connection pool");
        let vault_client_future = vault::Client::from_env();
        let vault_client = block_on(vault_client_future).expect("Unable to create vault client");

        (
            Arc::new(private_key),
            public_key,
            root_node,
            Arc::new(db_pool),
            Arc::new(vault_client),
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

    fn create_test_citizen_in_db(
        db: &db::DbPooledConnection,
    ) -> (String, ed25519_dalek::Keypair, ed25519_dalek::Keypair) {
        use crate::db::models::{Citizen, NewCitizen};

        let identifier = random_string(64);
        let access_keypair = key_utils::gen_ed25519_keypair();
        let keypair_ed25519 = key_utils::gen_ed25519_keypair();
        let private_x25519 = key_utils::gen_x25519_static_secret();
        let public_x25519 = x25519_dalek::PublicKey::from(&private_x25519);

        let private_secret_key = orion::aead::SecretKey::generate(32).unwrap();
        let aead_data = orion::aead::seal(&private_secret_key, b"secret").unwrap();

        let new_citizen = NewCitizen {
            identifier: &identifier,
            access_key: &base64::encode_config(
                access_keypair.public.as_bytes(),
                base64::STANDARD_NO_PAD,
            ),
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

        (identifier, access_keypair, keypair_ed25519)
    }

    #[test]
    fn test_chatrouille_empty() {
        let (private_key, _, root_node, db_pool, vault_client) = setup_chatrouille();

        // Empty
        let request = Request::builder().body(Body::empty()).unwrap();
        let response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body_contains(response, "too small"));
    }

    #[test]
    fn test_chatrouille_random() {
        use rand::prelude::*;

        let (private_key, _, root_node, db_pool, vault_client) = setup_chatrouille();

        // Random data
        let mut random_data = [0_u8; 256];
        rand::thread_rng().fill_bytes(&mut random_data);
        // Make sure the prefix is always invalid
        random_data[0] = 0;

        let request = Request::builder()
            .body(Body::from(random_data.to_vec()))
            .unwrap();
        let response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body_contains(response, "prefix is invalid"));
    }

    #[test]
    fn test_chatrouille_wrong_public_key() {
        let (private_key, _, root_node, db_pool, vault_client) = setup_chatrouille();
        let another_private_key = key_utils::gen_private_key();
        let another_public_key = key_utils::gen_public_key(&another_private_key);

        // Valid empty query, but with a wrong public key :-)
        let query = chatrouille::pack_unsigned_query(&[], &another_public_key).unwrap();

        let request = Request::builder().body(Body::from(query.0)).unwrap();

        let response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body_contains(response, "Unable to decrypt"));
    }

    #[test]
    fn test_chatrouille_wrong_graphql() {
        let (private_key, public_key, root_node, db_pool, vault_client) = setup_chatrouille();

        let query = chatrouille::pack_unsigned_query(
            &serde_json::to_vec(&json!({
              "not graphql": true
            }))
            .unwrap(),
            &public_key,
        )
        .unwrap();
        let request = Request::builder().body(Body::from(query.0)).unwrap();
        let response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(body_contains(response, "missing field"));
    }

    #[test]
    fn test_chatrouille_valid_unsigned() {
        let (private_key, public_key, root_node, db_pool, vault_client) = setup_chatrouille();
        let timestamp = get_timestamp().unwrap();

        let (query, shared_secret) = chatrouille::pack_unsigned_query(
      &serde_json::to_vec(&json!({
        "graphql": {
          "operationName": "loadCitizenPublicKey",
          "variables": {
            "identifier": "abcdef"
          },
          "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicX25519Dalek publicEd25519Dalek }}"
        },
        "exp": timestamp+60,
      }))
      .unwrap(),
      &public_key,
    )
    .unwrap();
        let request = Request::builder().body(Body::from(query)).unwrap();
        let encrypted_response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
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
        let (private_key, public_key, root_node, db_pool, vault_client) = setup_chatrouille();
        let timestamp = get_timestamp().unwrap();

        let (query, _) = chatrouille::pack_unsigned_query(
      &serde_json::to_vec(&json!({
        "graphql": {
          "operationName": "loadCitizenPublicKey",
          "variables": {
            "identifier": "abcdef"
          },
          "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicX25519Dalek publicEd25519Dalek }}"
        },
        "citizenIdentifier": "canard",
        "exp": timestamp+60,
      }))
      .unwrap(),
      &public_key,
    )
    .unwrap();

        let request = Request::builder().body(Body::from(query)).unwrap();
        let response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(body_contains(response, "signed"));
    }
    #[test]
    fn test_chatrouille_unvalid_expired() {
        let (private_key, public_key, root_node, db_pool, vault_client) = setup_chatrouille();
        let timestamp = get_timestamp().unwrap();

        let (query, _) = chatrouille::pack_unsigned_query(
      &serde_json::to_vec(&json!({
        "graphql": {
          "operationName": "loadCitizenPublicKey",
          "variables": {
            "identifier": "abcdef"
          },
          "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicX25519Dalek publicEd25519Dalek }}"
        },
        "exp": timestamp-60,
      }))
      .unwrap(),
      &public_key,
    )
    .unwrap();

        let request = Request::builder().body(Body::from(query)).unwrap();
        let response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
        assert_eq!(response.status(), StatusCode::GONE);
        assert!(body_contains(response, "expired"));
    }
    #[test]
    fn test_chatrouille_valid_signed() {
        let (private_key, public_key, root_node, db_pool, vault_client) = setup_chatrouille();

        let db = db_pool.get().expect("Database connection failed");
        let (identifier, access_keypair, keypair) = create_test_citizen_in_db(&db);
        let timestamp = get_timestamp().unwrap();

        let (query, shared_secret) = chatrouille::pack_signed_query(
      &serde_json::to_vec(&json!({
        "graphql": {
          "operationName": "loadCitizenPublicKey",
          "variables": {
            "identifier": identifier
          },
          "query": "query loadCitizenPublicKey($identifier: String!) { loadCitizenPublicKeys(identifier: $identifier) { publicEd25519Dalek }}"
        },
        "citizenIdentifier": identifier,
        "exp": timestamp + 60,
      }))
      .unwrap(),
      &public_key,
      &access_keypair,
    )
    .unwrap();
        let request = Request::builder().body(Body::from(query)).unwrap();
        let encrypted_response = block_on(chatrouille(
            request,
            root_node,
            db_pool,
            vault_client,
            private_key,
        ))
        .unwrap();
        assert_eq!(encrypted_response.status(), StatusCode::OK);
        let encrypted_body = read_response_body(encrypted_response);
        let response = chatrouille::unpack_response(&encrypted_body, &shared_secret).unwrap();

        let response_text = std::str::from_utf8(&response).unwrap();
        let ed25519_dalek_base64 =
            &base64::encode_config(keypair.public.as_bytes(), base64::STANDARD_NO_PAD);

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
