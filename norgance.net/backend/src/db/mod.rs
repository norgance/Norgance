use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;

//use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use snafu::{ResultExt, Snafu};

pub mod models;
pub mod schema;

#[derive(Debug, Snafu)]
pub enum NorganceDatabaseError {
  #[snafu(display("Setting not found: {}", source))]
  SettingNotFound { source: std::env::VarError },

  #[snafu(display("Wrong setting: {}", source))]
  WrongSetting { source: std::num::ParseIntError },

  #[snafu(display("Error while creating database pool: {}", source))]
  DatabasePoolCreation { source: r2d2::Error },
  #[snafu(display("Error while migrating database: {}", source))]
  DatabaseMigrations {
    source: diesel_migrations::RunMigrationsError,
  },
  #[snafu(display("Error while querying the database database: {}", source))]
  QueryError { source: diesel::result::Error },

  #[snafu(display("Error while decoding base64: {}", source))]
  Base64Error { source: base64::DecodeError },

  #[snafu(display("Error while loading public key: {}", source))]
  Ed25519Error {
    source: ed25519_dalek::SignatureError,
  },
}

pub type Result<T, E = NorganceDatabaseError> = std::result::Result<T, E>;

pub type DbConnection = diesel::PgConnection;
pub type DbPool = diesel::r2d2::Pool<ConnectionManager<DbConnection>>;
pub type DbPooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<DbConnection>>;

pub fn create_connection_pool() -> Result<DbPool> {
  let database_url = std::env::var("DATABASE_URL").context(SettingNotFound)?;
  let database_max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
    .unwrap_or_else(|_| String::from("16"))
    .parse::<u32>()
    .context(WrongSetting)?;

  let manager = ConnectionManager::<PgConnection>::new(database_url);
  let pool = diesel::r2d2::Builder::new()
    .max_size(database_max_connections)
    .min_idle(Some(1))
    .build(manager)
    .context(DatabasePoolCreation)?;
  //let pool = DbPool::new(manager).context(DatabasePoolCreation)?;
  Ok(pool)
}

pub fn migrate(connection: &DbConnection) -> Result<()> {
  diesel_migrations::run_pending_migrations(connection).context(DatabaseMigrations)?;
  Ok(())
}

pub fn is_identifier_available(db: &DbPooledConnection, input_identifier: &str) -> Result<bool> {
  use diesel::dsl::*;
  use diesel::prelude::*;
  use schema::citizens::dsl::*;

  let query = select(not(exists(
    citizens
      .filter(identifier.eq(input_identifier))
      .select(identifier),
  )));

  //println!("{}", diesel::debug_query::<diesel::pg::Pg, _>(&query));

  let result = query.get_result(db).context(QueryError)?;

  Ok(result)
}

pub fn load_citizen_personal_data(
  db: &DbPooledConnection,
  input_identifier: &str,
) -> Result<Option<String>> {
  use diesel::prelude::*;
  use schema::citizens::dsl::*;

  let query = citizens
    .filter(identifier.eq(input_identifier))
    .select(aead_data)
    .limit(1);

  // println!("{}", diesel::debug_query::<diesel::pg::Pg, _>(&query));

  let result = query.load::<String>(db).context(QueryError)?.pop();

  Ok(result)
}

pub fn load_citizen_public_keys(
  db: &DbPooledConnection,
  input_identifier: &str,
) -> Result<Option<models::CitizenPublicKeys>> {
  use diesel::prelude::*;
  use schema::citizens::dsl::*;

  let result = citizens
    .filter(identifier.eq(input_identifier))
    .select((public_x25519_dalek, public_ed25519_dalek))
    .limit(1)
    .load::<models::CitizenPublicKeys>(db)
    .context(QueryError)?
    .pop();

  Ok(result)
}

pub fn load_citizen_access_key(
  db: &DbPooledConnection,
  input_identifier: &str,
) -> Result<Option<ed25519_dalek::PublicKey>> {
  use diesel::prelude::*;
  use schema::citizens::dsl::*;

  let result = citizens
    .filter(identifier.eq(input_identifier))
    .select(access_key)
    .limit(1)
    .load::<String>(db)
    .context(QueryError)?
    .pop();

  let result_bytes = match result {
    Some(r) => base64::decode(r).context(Base64Error)?,
    None => return Ok(None),
  };

  let public_key = ed25519_dalek::PublicKey::from_bytes(&result_bytes).context(Ed25519Error)?;

  Ok(Some(public_key))
}

pub fn health_check(db: &DbPooledConnection) -> Result<()> {
  use diesel::prelude::*;

  diesel::sql_query("SELECT 1;")
    .execute(db)
    .context(QueryError)?;

  Ok(())
}
