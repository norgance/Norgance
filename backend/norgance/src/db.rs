use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;

//use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use snafu::{ResultExt, Snafu};

use crate::schema;

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
}

pub type Result<T, E = NorganceDatabaseError> = std::result::Result<T, E>;

pub type DbConnection = diesel::PgConnection;
pub type DbPool = diesel::r2d2::Pool<ConnectionManager<DbConnection>>;
pub type DbPooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<DbConnection>>;

pub fn create_connection_pool() -> Result<DbPool> {
  let database_url = std::env::var("DATABASE_URL").context(SettingNotFound)?;
  let database_max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
    .unwrap_or(String::from("16"))
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

pub fn is_identifier_available(
  db: &DbPooledConnection,
  requested_identifier: &str,
) -> Result<bool> {
  use diesel::dsl::*;
  use diesel::prelude::*;
  use schema::citizens::dsl::*;

  let query = select(not(exists(
    citizens
      .filter(identifier.eq(requested_identifier))
      .select(identifier),
  )));

  //println!("{}", diesel::debug_query::<diesel::pg::Pg, _>(&query));

  let result = query.get_result(db).context(QueryError)?;

  Ok(result)
}

pub fn load_citizen_public_keys(
  db: &DbPooledConnection,
  requested_identifier: &str,
) -> Result<Option<crate::models::CitizenPublicKeys>> {
  use crate::schema::citizens::dsl::*;
  use diesel::prelude::*;

  let result = citizens
    .filter(identifier.eq(requested_identifier))
    .select((public_x448, public_x25519_dalek, public_ed25519_dalek))
    .limit(1)
    .load::<crate::models::CitizenPublicKeys>(db)
    .context(QueryError)?
    .pop();

  Ok(result)
}
