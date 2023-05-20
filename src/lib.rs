use diesel::pg::PgConnection;
use diesel::r2d2;

pub type DbConnection = PgConnection;
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<DbConnection>>;

pub fn create_connection_pool() -> DbPool {
    let config = config::get_config().unwrap();

    let manager = r2d2::ConnectionManager::<DbConnection>::new(&config.database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path")
}

#[cfg(test)]
pub fn create_test_connection_pool() -> DbPool {
    let config = config::get_config().unwrap();

    let manager = r2d2::ConnectionManager::<DbConnection>::new(&config.test_database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path");
    let mut conn = pool.get().unwrap();
    run_migrations(&mut conn).unwrap();
    pool
}

#[cfg(test)]
fn run_migrations(connection: &mut impl diesel_migrations::MigrationHarness<diesel::pg::Pg>) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations};
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    connection.revert_all_migrations(MIGRATIONS).unwrap();
    connection.run_pending_migrations(MIGRATIONS).unwrap();
    Ok(())
}

pub mod config;
pub mod models;
pub mod schema;
pub mod services;
pub mod swagger;
pub mod middleware;
pub mod errors;
pub mod traits;
