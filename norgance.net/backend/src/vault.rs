use std::env;
use hashicorp_vault::client::{VaultClient,TokenData};
use snafu::{ResultExt, OptionExt, Snafu};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Snafu)]
pub enum VaultError {
  MissingVaultAddr { source: env::VarError },
  MissingVaultToken { source: env::VarError },
  InvalidClient { source: hashicorp_vault::Error },
  CannotGetSecret { source: hashicorp_vault::Error },
  Base64Decode { source: base64::DecodeError },
  SecretLoad,
}

pub type Result<T, E = VaultError> = std::result::Result<T, E>;
pub type Client = VaultClient<TokenData>;

pub fn make_client() -> Result<Client>{
  let vault_addr = env::var("VAULT_ADDR").context(MissingVaultAddr)?;
  let vault_token = env::var("VAULT_TOKEN").context(MissingVaultToken)?;

  Ok(VaultClient::new(vault_addr, vault_token).context(InvalidClient)?)
}

pub fn get_private_key(client: &Client) -> Result<x448::Secret> {
  let secret_path =
  env::var("VAULT_PRIVATE_KEY_PATH").unwrap_or_else(|_| String::from("private_key"));

  let secret_base64 = client.get_secret(secret_path).context(CannotGetSecret)?;
  let secret_bytes = base64::decode(secret_base64).context(Base64Decode)?;

  let secret = x448::Secret::from_bytes(&secret_bytes).context(SecretLoad)?;
  Ok(secret)
}
