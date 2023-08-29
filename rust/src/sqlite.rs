use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
pub use sqlx::{Pool, Sqlite};

use std::str::FromStr as _;

use crate::StartError;

pub async fn init_database_pool(url: &str) -> Result<Pool<Sqlite>, StartError> {
    Ok(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(SqliteConnectOptions::from_str(url)?.pragma("foreign_keys", "1"))
        .await?)
}

pub async fn migrate(pool: &Pool<Sqlite>) -> Result<(), StartError> {
    sqlx::migrate!().run(pool).await?;
    Ok(())
}
