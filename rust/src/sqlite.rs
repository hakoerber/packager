use std::time;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;
pub use sqlx::{Pool, Sqlite};

use std::str::FromStr as _;

use crate::StartError;

pub async fn init_database_pool(url: &str) -> Result<Pool<Sqlite>, StartError> {
    Ok(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::from_str(url)?
                .log_statements(log::LevelFilter::Debug)
                .log_slow_statements(log::LevelFilter::Warn, time::Duration::from_millis(100))
                .pragma("foreign_keys", "1"),
        )
        .await?)
}

pub async fn migrate(url: &str) -> Result<(), StartError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::from_str(url)?
                .pragma("foreign_keys", "0")
                .log_statements(log::LevelFilter::Debug),
        )
        .await?;

    sqlx::migrate!().run(&pool).await?;
    Ok(())
}
