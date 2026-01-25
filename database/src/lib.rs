use base64::Engine as _;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgQueryResult;
use std::fmt;

pub mod error;
mod macros;
pub mod postgres;
pub mod types;

pub use error::{DataError, Error, InitError, QueryError};

pub trait Database {
    type Pool;

    fn init_database_pool(
        url: &str,
    ) -> impl std::future::Future<Output = Result<Self::Pool, InitError>> + Send;

    fn migrate(url: &str) -> impl std::future::Future<Output = Result<(), InitError>> + Send;
}

pub type DB = self::postgres::DB;
pub type Pool = sqlx::Pool<sqlx::Postgres>;

pub enum QueryType {
    Insert,
    Update,
    Select,
    Delete,
}

impl fmt::Display for QueryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Insert => "insert",
                Self::Update => "update",
                Self::Select => "select",
                Self::Delete => "delete",
            }
        )
    }
}

pub struct QueryClassification<Component>
where
    Component: ToString,
{
    pub query_type: QueryType,
    pub component: Component,
}

pub fn sqlx_query<Component>(
    classification: &QueryClassification<Component>,
    query: &str,
    labels: &[(&'static str, String)],
) where
    Component: ToString,
{
    let query_id = {
        let mut hasher = Sha256::new();
        hasher.update(query);
        hasher.finalize()
    };

    // 9 bytes is enough to be unique
    // If this is divisible by 3, it means that we can base64-encode it without
    // any "=" padding
    //
    // cannot panic, as the output for sha256 will always be bit
    let query_id = &query_id[..9];

    let query_id = base64::engine::general_purpose::STANDARD.encode(query_id);
    let mut labels = Vec::from(labels);
    labels.extend_from_slice(&[
        ("query_id", query_id),
        ("query_type", classification.query_type.to_string()),
        ("query_component", classification.component.to_string()),
    ]);
    metrics::counter!("packager_database_queries_total", &labels).increment(1);
}

pub fn sqlx_query_file<Component>(
    classification: &QueryClassification<Component>,
    path: &str,
    labels: &[(&'static str, String)],
) where
    Component: ToString,
{
    let query_id = {
        let mut hasher = Sha256::new();
        hasher.update(path);
        hasher.finalize()
    };

    // 9 bytes is enough to be unique
    // If this is divisible by 3, it means that we can base64-encode it without
    // any "=" padding
    //
    // cannot panic, as the output for sha256 will always be bit
    let query_id = &query_id[..9];

    let query_id = base64::engine::general_purpose::STANDARD.encode(query_id);
    let mut labels = Vec::from(labels);
    labels.extend_from_slice(&[
        ("query_id", query_id),
        ("query_type", classification.query_type.to_string()),
        ("query_component", classification.component.to_string()),
    ]);
    metrics::counter!("packager_database_queries_total", &labels).increment(1);
}

#[macro_export]
macro_rules! query_many_to_many_single {
    ( $class:expr, $pool:expr, $struct_row:path, $struct_rows:path, $struct_into:path, $error_type:path, $query:expr, $( $args:tt )* ) => {
        {
            use tracing::Instrument as _;
            use futures::TryStreamExt as _;
            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: Vec<$struct_row> = sqlx::query_as!(
                    $struct_row,
                    $query,
                    $( $args )*
                )
                .fetch($pool)
                .try_collect::<Vec<$struct_row>>()
                .await?;

                if result.is_empty() {
                    Ok(None)
                } else {
                    let out: $struct_rows = result.into();
                    let out: $struct_into = <_ as TryInto<$struct_into>>::try_into(out)?;
                    Ok::<_, $error_type>(Some(out))
                }

            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! query_one {
    ( $class:expr, $pool:expr, $struct_row:path, $struct_into:path, $error_type:path, $query:expr, $( $args:tt )*) => {

        {
            use tracing::Instrument as _;

            async {
                $crate::sqlx_query($class, $query, &[]);
                let result: Result<Option<$struct_into>, $error_type> = sqlx::query_as!(
                    $struct_row,
                    $query,
                    $( $args )*
                )
                .fetch_optional($pool)
                .await?
                .map(|row: $struct_row| <_ as TryInto<$struct_into>>::try_into(row))
                .transpose();

                result

            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! query_one_file {
    ( $class:expr, $pool:expr, $struct_row:path, $struct_into:path, $error_type:path, $path:literal, $( $args:tt )*) => {

        {
            use tracing::Instrument as _;

            async {
                $crate::sqlx_query_file($class, $path, &[]);
                let result: Result<Option<$struct_into>, $error_type> = sqlx::query_file_as!(
                    $struct_row,
                    $path,
                    $( $args )*
                )
                .fetch_optional($pool)
                .await?
                .map(|row: $struct_row| <_ as TryInto<$struct_into>>::try_into(row))
                .transpose();

                result

            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! strip_plus {
    (+ $($rest:tt)*) => {
        $($rest)*
    }
}

// #[macro_export]
// macro_rules! execute_unchecked {
//     ( $class:expr, $pool:expr, $error_type:path, $query:expr, $( $args:expr ),* $(,)? ) => {{
//         use tracing::Instrument as _;
//         async {
//             $crate::sqlx_query($class, $query, &[]);
//             let query = sqlx::query($query);

//             $(
//                 let query = query.bind($args);
//             )*

//             let result: Result<sqlx::postgres::PgQueryResult, $error_type> =
//                 query.execute($pool).await.map_err(|e| e.into());

//             result
//         }
//         .instrument(tracing::info_span!("packager::sql::query", "query"))
//     }};
// }

pub struct QueryResult {
    rows_affected: u64,
}

impl QueryResult {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl From<PgQueryResult> for QueryResult {
    fn from(value: PgQueryResult) -> Self {
        Self {
            rows_affected: value.rows_affected(),
        }
    }
}
