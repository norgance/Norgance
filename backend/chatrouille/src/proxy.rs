extern crate hyper;
extern crate anyhow;

use hyper::{Body, Client, Method, Request, Uri};

#[derive(thiserror::Error, Debug)]
pub enum ProxyError {
    #[error("Error while building the query")]
    Builder(#[source] hyper::http::Error),

    #[error("Unable to do the diffie Hellman")]
    Fuck,
    
    #[error("Not OK HTTP response code")]
    NotOK,
    
    #[error("Request error")]
    Request(#[source] hyper::Error),

    #[error("Body reading error")]
    BodyRead(#[source] hyper::Error),
}

pub async fn proxy() -> Result<Vec<u8>, ProxyError> {
  /*let client = Client::new();

  let uri = "http://httpbin.org/ip".parse()?;
  let resp = client.get(uri).await?;*/

  let req = match Request::builder()
    .method(Method::POST)
    .uri("http://httpbin.org/post")
    .header("content-type", "application/json")
    .body(Body::from(r#"{"library":"hyper"}"#)) {
      Ok(req) => req,
      Err(error) => return Err(ProxyError::Builder(error)),
    };

  let client = Client::new();

  // POST it...
  let resp = match client.request(req).await {
    Ok(resp) => resp,
    Err(error) => return Err(ProxyError::Request(error)),
  };
  println!("Response: {}", resp.status());

  if resp.status() != hyper::StatusCode::OK {
    return Err(ProxyError::NotOK);
  }

  let body = match hyper::body::to_bytes(resp.into_body()).await {
    Ok(body) => body,
    Err(error) => return Err(ProxyError::BodyRead(error)),
  };

  return Ok(body.to_vec());
}
