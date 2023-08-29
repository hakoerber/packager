use std::time;

use base64::Engine as _;
use sha2::{Digest, Sha256};
use tracing::Instrument;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;
pub use sqlx::{Pool, Sqlite};

use std::str::FromStr as _;

use crate::StartError;

#[tracing::instrument]
pub async fn init_database_pool(url: &str) -> Result<Pool<Sqlite>, StartError> {
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

pub fn sqlx_query(query: &str, labels: &[(&'static str, String)]) {
    let query_id = {
        let mut hasher = Sha256::new();
        hasher.update(query);
        hasher.finalize()
    };
    let query_id = base64::engine::general_purpose::STANDARD.encode(query_id);
    let mut labels = Vec::from(labels);
    labels.push(("query_id", query_id));
    metrics::counter!("packager_database_queries_total", 1, &labels)
}

#[macro_export]
macro_rules! query_all {
    ( $pool:expr, $struct_row:path, $struct_into:path, $query:expr, $( $args:tt )* ) => {
        async {
            crate::sqlite::sqlx_query($query, &[]);
            let result: Result<Vec<$struct_into>, Error> = sqlx::query_as!(
                $struct_row,
                $query,
                $( $args )*
            )
            .fetch($pool)
            .map_ok(|row: $struct_row| row.try_into())
            .try_collect::<Vec<Result<$struct_into, Error>>>()
            .await?
            .into_iter()
            .collect::<Result<Vec<$struct_into>, Error>>();

            result

        }.instrument(tracing::info_span!("packager::sql::query", "query"))
    };
}

#[macro_export]
macro_rules! query_one {
    ( $pool:expr, $struct_row:path, $struct_into:path, $query:expr, $( $args:tt )*) => {
        async {
            let result: Result<Option<$struct_into>, Error> = sqlx::query_as!(
                $struct_row,
                $query,
                $( $args )*
            )
            .fetch_optional($pool)
            .await?
            .map(|row: $struct_row| row.try_into())
            .transpose();

            result

        }.instrument(tracing::info_span!("packager::sql::query", "query"))
    };
}

#[macro_export]
macro_rules! query_exists {
    ( $pool:expr, $query:expr, $( $args:tt )*) => {
        async {
            let result: bool = sqlx::query!(
                $query,
                $( $args )*
            )
            .fetch_optional($pool)
            .await?
            .is_some();

            Ok(result)

        }.instrument(tracing::info_span!("packager::sql::query", "query"))
    };
}

#[macro_export]
macro_rules! execute {
    ( $pool:expr, $query:expr, $( $args:tt )*) => {
        async {
            let result: Result<sqlx::sqlite::SqliteQueryResult, Error> = sqlx::query!(
                $query,
                $( $args )*
            )
            .execute($pool)
            .await
            .map_err(|e| e.into());

            result


        }.instrument(tracing::info_span!("packager::sql::query", "query"))
    };
}

#[macro_export]
macro_rules! execute_returning {
    ( $pool:expr, $query:expr, $t:path, $fn:expr, $( $args:tt )*) => {
        async {
            let result: Result<$t, Error> = sqlx::query!(
                $query,
                $( $args )*
            )
            .fetch_one($pool)
            .map_ok($fn)
            .await
            .map_err(Into::into);

            result


        }.instrument(tracing::info_span!("packager::sql::query", "query"))
    };
}

#[macro_export]
macro_rules! execute_returning_uuid {
    ( $pool:expr, $query:expr, $( $args:tt )*) => {
        async {
            let result: Result<Uuid, Error> = sqlx::query!(
                $query,
                $( $args )*
            )
            .fetch_one($pool)
            .map_ok(|row| Uuid::try_parse(&row.id))
            .await?
            .map_err(Into::into);

            result


        }.instrument(tracing::info_span!("packager::sql::query", "query"))
    };
}
