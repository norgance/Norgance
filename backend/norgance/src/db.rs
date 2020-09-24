use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;

//use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum NorganceDatabaseError {
  #[snafu(display("Setting not found: {}", source))]
  SettingNotFound { source: std::env::VarError },

  #[snafu(display("Wrong setting: {}", source))]
  WrongSetting { source: std::num::ParseIntError },

  #[snafu(display("Error while creating database pool: {}", source))]
  DatabasePoolCreation { source: r2d2::Error },
  
  #[snafu(display("Error while migrating database: {}", source))]
  DatabaseMigrations { source: diesel_migrations::RunMigrationsError },
}

pub type Result<T, E = NorganceDatabaseError> = std::result::Result<T, E>;

pub type DbConnection = diesel::PgConnection;
pub type DbPool = diesel::r2d2::Pool<ConnectionManager<DbConnection>>;

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
