use std::time;

use tracing::Instrument;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;
pub use sqlx::{Pool as SqlitePool, Sqlite};

use std::str::FromStr as _;

pub use sqlx::Type;

use crate::StartError;

pub fn int_to_bool(value: i32) -> bool {
    match value {
        0 => false,
        1 => true,
        _ => panic!("got invalid boolean from sqlite"),
    }
}

#[tracing::instrument]
pub async fn init_database_pool(url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, StartError> {
    async {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                SqliteConnectOptions::from_str(url)?
                    .log_statements(log::LevelFilter::Debug)
                    .log_slow_statements(log::LevelFilter::Warn, time::Duration::from_millis(100))
                    .pragma("foreign_keys", "1"),
            )
            .await
    }
    .instrument(tracing::info_span!("packager::sql::pool"))
    .await
    .map_err(Into::into)
}

#[tracing::instrument]
pub async fn migrate(url: &str) -> Result<(), StartError> {
    async {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(
                SqliteConnectOptions::from_str(url)?
                    .pragma("foreign_keys", "0")
                    .log_statements(log::LevelFilter::Debug),
            )
            .await?;

        sqlx::migrate!().run(&pool).await
    }
    .instrument(tracing::info_span!("packager::sql::migrate"))
    .await?;

    Ok(())
}
