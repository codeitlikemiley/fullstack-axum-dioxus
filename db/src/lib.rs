use std::str::FromStr;

pub use cornucopia_async::Params;
pub use deadpool_postgres::{Pool, PoolError, Transaction};
pub use tokio_postgres::Error as TokioPostgresError;

pub fn create_pool(database_url: &str) -> deadpool_postgres::Pool {
    let config = tokio_postgres::Config::from_str(database_url).unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
    deadpool_postgres::Pool::builder(manager).build().unwrap()
}

// Placeholder User type when database queries are not generated
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
}

// Note: Set DATABASE_URL environment variable to generate database queries
// If DATABASE_URL is not set, the build script creates an empty cornucopia.rs
// to avoid compilation errors when trying to access generated query code