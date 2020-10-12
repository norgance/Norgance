use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum PasswordQualityError {
  #[snafu(display("Invalid Prefix"))]
  InvalidPrefix,

  #[snafu(display("Query error"))]
  QueryError { source: reqwest::Error },

  #[snafu(display("Result parsing error"))]
  ResultParsingError { source: reqwest::Error },
}

#[derive(juniper::GraphQLObject, serde::Deserialize, Clone, Debug)]
pub struct PasswordQuality {
  suffix: String,
  quality: String,
}

fn validate_prefix(prefix: &str) -> std::result::Result<(), PasswordQualityError> {
  lazy_static! {
    static ref VALID_PREFIX: regex::Regex =
      regex::Regex::new("^[a-fA-F0-9]{5}$").expect("Unable to build validate_prefix regex");
  }

  if VALID_PREFIX.is_match(prefix) {
    Ok(())
  } else {
    Err(PasswordQualityError::InvalidPrefix)
  }
}

#[derive(serde::Deserialize, Clone, Debug)]
struct PasswordQualityAPIResponse {
  suffixes: Vec<PasswordQuality>,
}

pub async fn check_password_quality(
  prefix: String,
) -> std::result::Result<Vec<PasswordQuality>, PasswordQualityError> {
  validate_prefix(&prefix)?;

  lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
      .timeout(std::time::Duration::new(30, 0))
      .build()
      .expect("Unable to build check_password_quality client");
    static ref API_PATH: String = std::env::var("PASSWORD_QUALITY_API_PATH")
      .unwrap_or_else(|_| String::from("http://localhost:3030/password_quality/"));
  }

  let response = CLIENT
    .get(&format!("{}{}.json", API_PATH.as_str(), &prefix))
    .send()
    .await
    .context(QueryError)?
    .json::<PasswordQualityAPIResponse>()
    .await
    .context(ResultParsingError)?;

  Ok(response.suffixes)
}

#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]
#[cfg(test)]
mod tests {
  use super::*;
  use tokio_test::block_on;

  #[test]
  fn test_check_password_quality() {
    block_on(check_password_quality(String::from("c92c8"))).unwrap();

    assert!(block_on(check_password_quality(String::from(""))).is_err());
    assert!(block_on(check_password_quality(String::from("ZZZZZ"))).is_err());
    assert!(block_on(check_password_quality(String::from("abcdef"))).is_err());
  }
}
