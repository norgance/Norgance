use snafu::{OptionExt, ResultExt, Snafu};
use std::env;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Snafu)]
pub enum VaultError {
  MissingVaultAddr {
    source: env::VarError,
  },
  MissingVaultCredentials {
    source: env::VarError,
  },
  //InvalidClient { source: hashicorp_vault::Error },
  CannotGetSecret,
  #[snafu(display("Base64 decoding: {}", source))]
  Base64Decode {
    source: base64::DecodeError,
  },
  SecretLoad,
  #[snafu(display("Client build: {}", source))]
  ClientBuild {
    source: reqwest::Error,
  },
  #[snafu(display("Query error: {}", source))]
  QueryError {
    source: reqwest::Error,
  },
  #[snafu(display("Result parsing error: {}", source))]
  ResultParsingError {
    source: reqwest::Error,
  },
  WrongCredentials,
}

pub type Result<T, E = VaultError> = std::result::Result<T, E>;

pub struct Client {
  addr: String,
  username: String,
  password: String,
  client: reqwest::Client,
  authentication: String,
}

impl Client {
  pub fn new(addr: &str, username: &str, password: &str) -> Result<Client> {
    let client = reqwest::ClientBuilder::new()
      .timeout(std::time::Duration::new(30, 0))
      .build()
      .context(ClientBuild)?;

    Ok(Client {
      addr: String::from(addr),
      authentication: String::new(),
      client,
      username: String::from(username),
      password: String::from(password),
    })
  }

  pub async fn from_env() -> Result<Client> {
    let vault_addr = env::var("VAULT_ADDR").context(MissingVaultAddr)?;
    let vault_credentials = env::var("VAULT_CREDENTIALS").context(MissingVaultCredentials)?;

    let op: Vec<&str> = vault_credentials.split(':').collect();
    let username = op.get(0).context(WrongCredentials)?;
    let password = op.get(1).context(WrongCredentials)?;

    let mut client = Client::new(&vault_addr, &username, &password)?;
    client.login().await?;

    Ok(client)
  }

  pub async fn login(&mut self) -> Result<()> {
    let response = self
      .client
      .post(&format!(
        "{}/v1/auth/userpass/login/{}",
        &self.addr,
        percent_encoding::utf8_percent_encode(&self.username, percent_encoding::NON_ALPHANUMERIC,)
          .to_string(),
      ))
      .json(&LoginPayload {
        password: String::from(&self.password),
      })
      .send()
      .await
      .context(QueryError)?
      .json::<LoginResponse>()
      .await
      .context(ResultParsingError)?;

    let token = response.auth.client_token;
    let bearer = String::from("Bearer ") + &token;
    self.authentication = bearer;
    Ok(())
  }

  async fn load_secret(&mut self, path: &str) -> Result<SecretDataResponse> {
    let response = self
      .client
      .get(&format!(
        "{}/v1/secret/data/{}",
        &self.addr,
        percent_encoding::utf8_percent_encode(path, percent_encoding::NON_ALPHANUMERIC).to_string()
      ))
      .header(reqwest::header::AUTHORIZATION, self.authentication.clone())
      .send()
      .await
      .context(QueryError)?
      .json::<SecretResponse>()
      .await
      .context(ResultParsingError)?;

    Ok(response.data)
  }

  pub async fn load_server_private_key(&mut self) -> Result<x448::Secret> {
    let secret_path =
      env::var("VAULT_PRIVATE_KEY_PATH").unwrap_or_else(|_| String::from("server_private_key"));

    let secret = self.load_secret(&secret_path).await?;

    let secret_base64 = secret.only_value().context(CannotGetSecret)?;

    let secret_bytes = base64::decode(secret_base64).context(Base64Decode)?;
    let secret = x448::Secret::from_bytes(&secret_bytes).context(SecretLoad)?;
    Ok(secret)  
  }
}

#[derive(serde::Serialize, Debug)]
struct LoginPayload {
  password: String,
}

#[derive(serde::Deserialize, Debug)]
struct LoginResponse {
  auth: LoginAuthResponse,
}

#[derive(serde::Deserialize, Debug)]
struct LoginAuthResponse {
  client_token: String,
}

#[derive(serde::Deserialize, Debug)]
struct SecretResponse {
  data: SecretDataResponse,
}

#[derive(serde::Deserialize, Debug)]
struct SecretDataResponse {
  data: std::collections::HashMap<String, String>,
}

impl SecretDataResponse {
  fn only_value(&self) -> Option<String> {
    if self.data.len() != 1 {
      return None;
    }
    match self.data.iter().next() {
      Some(op) => Some(op.1.into()),
      None => None,
    }
  }
}
