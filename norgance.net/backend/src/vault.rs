use snafu::{OptionExt, ResultExt, Snafu};
use std::env;
use std::sync::{Arc, RwLock};

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
    SecretX448Load,
    SecretEd25519Load {
        source: ed25519_dalek::ed25519::Error,
    },
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
    AuthenticationLock,
}

pub type Result<T, E = VaultError> = std::result::Result<T, E>;

pub struct Client {
    addr: String,
    username: String,
    password: String,
    client: reqwest::Client,
    authentication: Arc<RwLock<String>>,
}

impl Client {
    pub fn new(addr: &str, username: &str, password: &str) -> Result<Client> {
        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::new(30, 0))
            .build()
            .context(ClientBuild)?;

        Ok(Client {
            addr: String::from(addr),
            authentication: Arc::new(RwLock::new(String::new())),
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
                percent_encoding::utf8_percent_encode(
                    &self.username,
                    percent_encoding::NON_ALPHANUMERIC,
                )
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

        {
            let mut authentication = match self.authentication.write() {
                Ok(a) => a,
                Err(_) => return Err(VaultError::AuthenticationLock),
            };
            *authentication = bearer;
        }

        Ok(())
    }

    async fn load_secret(&self, path: &str) -> Result<SecretDataResponse> {
        let authentication : String;
        {
            let authentication_rwlock_guard = match self.authentication.read() {
                Ok(a) => a,
                Err(_) => return Err(VaultError::AuthenticationLock),
            };
            authentication = authentication_rwlock_guard.clone();
        }
        let response = self
            .client
            .get(&format!(
                "{}/v1/secret/data/{}",
                &self.addr,
                percent_encoding::utf8_percent_encode(path, percent_encoding::NON_ALPHANUMERIC)
                    .to_string()
            ))
            .header(
                reqwest::header::AUTHORIZATION,
                authentication,
            )
            .send()
            .await
            .context(QueryError)?
            .json::<SecretResponse>()
            .await
            .context(ResultParsingError)?;

        Ok(response.data)
    }

    pub async fn load_server_secrets(&self) -> Result<ServerPrivateSecrets> {
        use std::convert::TryInto;

        let secret_path =
            env::var("VAULT_PRIVATE_KEY_PATH").unwrap_or_else(|_| String::from("server_secrets"));

        let secrets_package = self.load_secret(&secret_path).await?;
        let secrets: ServerPrivateSecrets = secrets_package.data.try_into()?;
        Ok(secrets)
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
    data: SecretDataSecretsPackage,
}

#[derive(serde::Deserialize, Debug)]
struct SecretDataSecretsPackage {
    x448_private_key: String,
    ed25519_private_key: String,
}

impl SecretDataSecretsPackage {
    pub fn get_private_key_x448(&self) -> Result<x448::Secret> {
        let secret_bytes = base64::decode(&self.x448_private_key).context(Base64Decode)?;
        let secret = x448::Secret::from_bytes(&secret_bytes).context(SecretX448Load)?;
        Ok(secret)
    }
    pub fn get_keypair_ed25519(&self) -> Result<ed25519_dalek::Keypair> {
        let secret_bytes = base64::decode(&self.ed25519_private_key).context(Base64Decode)?;
        let secret =
            ed25519_dalek::SecretKey::from_bytes(&secret_bytes).context(SecretEd25519Load)?;
        let public: ed25519_dalek::PublicKey = (&secret).into();
        let keypair = ed25519_dalek::Keypair { public, secret };
        Ok(keypair)
    }
}

pub struct ServerPrivateSecrets {
    pub x448_private_key: x448::Secret,
    pub ed25519_keypair: ed25519_dalek::Keypair,
}

impl std::convert::TryFrom<SecretDataSecretsPackage> for ServerPrivateSecrets {
    type Error = VaultError;

    fn try_from(item: SecretDataSecretsPackage) -> Result<Self, Self::Error> {
        let x448_private_key = item.get_private_key_x448()?;
        let ed25519_keypair = item.get_keypair_ed25519()?;
        Ok(ServerPrivateSecrets {
            x448_private_key,
            ed25519_keypair,
        })
    }
}
