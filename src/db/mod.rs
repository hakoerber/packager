use base64::Engine as _;
use sha2::{Digest, Sha256};
use std::fmt;

pub(crate) mod error;
pub(crate) mod postgres;

use crate::StartError;

pub trait Database {
    type Pool;

    fn init_database_pool(
        url: &str,
    ) -> impl std::future::Future<Output = Result<Self::Pool, StartError>> + Send;
    fn migrate(url: &str) -> impl std::future::Future<Output = Result<(), StartError>> + Send;
}

pub type DB = self::postgres::DB;
pub(crate) type Pool = sqlx::Pool<sqlx::Postgres>;

pub(crate) enum QueryType {
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

pub(crate) enum Component {
    Inventory,
    User,
    Trips,
    Todo,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Inventory => "inventory",
                Self::User => "user",
                Self::Trips => "trips",
                Self::Todo => "todo",
            }
        )
    }
}

pub(crate) struct QueryClassification {
    pub query_type: QueryType,
    pub component: Component,
}

pub(crate) fn sqlx_query(
    classification: &QueryClassification,
    query: &str,
    labels: &[(&'static str, String)],
) {
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

// This does not work, as the query*! macros expect a string literal for the query, so
// it has to be there at compile time
//
// fn query_all<Row, Out>(
//     classification: &QueryClassification,
//     pool: &Pool,
//     query: &'static str,
//     args: &[&str],
// ) {
//     async {
//         sqlx_query(classification, query, &[]);
//         let result: Result<Vec<Out>, Error> = sqlx::query_as!(Row, query, args)
//             .fetch(pool)
//             .map_ok(|row: Row| row.try_into())
//             .try_collect::<Vec<Result<Out, Error>>>()
//             .await?
//             .into_iter()
//             .collect::<Result<Vec<Out>, Error>>();

//         result
//     }
//     .instrument(tracing::info_span!("packager::sql::query", "query"))
// }

#[macro_export]
macro_rules! query_all {
    ( $class:expr, $pool:expr, $struct_row:path, $struct_into:path, $query:expr, $( $args:tt )* ) => {
        {
            use tracing::Instrument as _;
            use futures::TryStreamExt as _;
            async {
                $crate::db::sqlx_query($class, $query, &[]);
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
        }
    };
}

#[macro_export]
macro_rules! query_many_to_many_single {
    ( $class:expr, $pool:expr, $struct_row:path, $struct_rows:path, $struct_into:path, $query:expr, $( $args:tt )* ) => {
        {
            use tracing::Instrument as _;
            use futures::TryStreamExt as _;
            async {
                $crate::db::sqlx_query($class, $query, &[]);
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
                    let out: $struct_into = out.try_into()?;
                    Ok::<_, $crate::error::Error>(Some(out))
                }

            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! query_one {
    ( $class:expr, $pool:expr, $struct_row:path, $struct_into:path, $query:expr, $( $args:tt )*) => {
        {
            use tracing::Instrument as _;

            async {
                $crate::db::sqlx_query($class, $query, &[]);
                let result: Result<Option<$struct_into>, $crate::error::Error> = sqlx::query_as!(
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
        }
    };
}

#[macro_export]
macro_rules! query_exists {
    ( $class:expr, $pool:expr, $query:expr, $( $args:tt )*) => {
        {
            use tracing::Instrument as _;
            async {
                $crate::db::sqlx_query($class, $query, &[]);
                let result: bool = sqlx::query!(
                    $query,
                    $( $args )*
                )
                .fetch_optional($pool)
                .await?
                .is_some();

                Ok(result)

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

#[macro_export]
macro_rules! execute_unchecked {
    ( $class:expr, $pool:expr, $query:expr, $( $args:expr ),* $(,)? ) => {{
        use tracing::Instrument as _;
        async {
            $crate::db::sqlx_query($class, $query, &[]);
            let query = sqlx::query($query);

            $(
                let query = query.bind($args);
            )*

            let result: Result<sqlx::postgres::PgQueryResult, Error> =
                query.execute($pool).await.map_err(|e| e.into());

            result
        }
        .instrument(tracing::info_span!("packager::sql::query", "query"))
    }};
}

#[macro_export]
macro_rules! execute {
    ( $class:expr, $pool:expr, $query:expr, $( $args:expr ),* $(,)? ) => {
        {
            use tracing::Instrument as _;
            async {
                $crate::db::sqlx_query($class, $query, &[]);
                let result: Result<sqlx::postgres::PgQueryResult, Error> = sqlx::query!(
                    $query,
                    $( $args ),*
                )
                .execute($pool)
                .await
                .map_err(|e| e.into());

                result
            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! execute_returning {
    ( $class:expr, $pool:expr, $query:expr, $t:path, $fn:expr, $( $args:expr ),* $(,)? ) => {
        {
            use tracing::Instrument as _;
            use futures::TryFutureExt as _;
            async {
                $crate::db::sqlx_query($class, $query, &[]);
                let result: Result<$t, Error> = sqlx::query!(
                    $query,
                    $( $args, )*
                )
                .fetch_one($pool)
                .map_ok($fn)
                .await
                .map_err(Into::into);

                result


            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}

#[macro_export]
macro_rules! execute_returning_uuid {
    ( $class:expr, $pool:expr, $query:expr, $( $args:expr ),* $(,)? ) => {
        {
            use tracing::Instrument as _;
            use futures::TryFutureExt as _;
            async {
                $crate::db::sqlx_query($class, $query, &[]);
                let result: Uuid = sqlx::query!(
                    $query,
                    $( $args ),*
                )
                .fetch_one($pool)
                .map_ok(|row| row.id)
                .await?;

                Ok(result)


            }.instrument(tracing::info_span!("packager::sql::query", "query"))
        }
    };
}
