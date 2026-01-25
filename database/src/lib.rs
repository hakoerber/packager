use std::fmt;

use sqlx::postgres::PgQueryResult;

mod error;
mod macros;
mod pool;
mod postgres;
pub mod telemetry;
pub mod types;

pub use error::{DataError, Error, InitError, QueryError};
pub use pool::Pool;
pub use telemetry::QueryClassification;

pub trait Database {
    type Pool;

    fn init_database_pool(
        url: &str,
    ) -> impl std::future::Future<Output = Result<Self::Pool, InitError>> + Send;

    fn migrate(url: &str) -> impl std::future::Future<Output = Result<(), InitError>> + Send;
}

pub type DB = self::postgres::DB;

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

pub struct QueryResult {
    rows_affected: u64,
}

impl QueryResult {
    #[must_use]
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
