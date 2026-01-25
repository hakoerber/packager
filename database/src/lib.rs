use std::{fmt, pin::Pin};

use futures::Stream;
use sqlx::{
    Acquire, Execute, Executor, Transaction,
    pool::PoolConnection,
    postgres::{PgQueryResult, PgRow, PgStatement},
};

pub mod error;
mod macros;
pub mod postgres;
pub mod telemetry;
pub mod types;

pub use error::{DataError, Error, InitError, QueryError};
pub use telemetry::QueryClassification;

pub trait Database {
    type Pool;

    fn init_database_pool(
        url: &str,
    ) -> impl std::future::Future<Output = Result<Self::Pool, InitError>> + Send;

    fn migrate(url: &str) -> impl std::future::Future<Output = Result<(), InitError>> + Send;
}

pub type DB = self::postgres::DB;

#[derive(Clone, Debug)]
pub struct Pool(sqlx::Pool<sqlx::Postgres>);

impl Pool {
    pub async fn begin(&self) -> Result<Transaction<'static, sqlx::Postgres>, Error> {
        todo!()
    }
}

impl<'c> Acquire<'c> for &Pool {
    type Database = sqlx::Postgres;
    type Connection = PoolConnection<sqlx::Postgres>;

    fn acquire(
        self,
    ) -> Pin<
        Box<
            dyn futures::Future<
                    Output = Result<sqlx::pool::PoolConnection<sqlx::Postgres>, sqlx::Error>,
                > + std::marker::Send
                + 'c,
        >,
    > {
        <&sqlx::Pool<sqlx::Postgres> as Acquire<'c>>::acquire(&self.0)
    }
    fn begin(
        self,
    ) -> Pin<
        Box<
            dyn futures::Future<Output = Result<Transaction<'c, sqlx::Postgres>, sqlx::Error>>
                + std::marker::Send
                + 'c,
        >,
    > {
        <&sqlx::Pool<sqlx::Postgres> as Acquire<'c>>::begin(&self.0)
    }
}

impl<'c> Executor<'c> for &Pool {
    type Database = sqlx::Postgres;

    fn fetch_many<'e, 'q, E>(
        self,
        query: E,
    ) -> Pin<
        Box<
            dyn Stream<Item = Result<sqlx::Either<PgQueryResult, PgRow>, sqlx::Error>>
                + std::marker::Send
                + 'e,
        >,
    >
    where
        'q: 'e,
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        <&sqlx::Pool<sqlx::Postgres> as Executor<'c>>::fetch_many(&self.0, query)
    }
    fn fetch_optional<'e, 'q, E>(
        self,
        query: E,
    ) -> Pin<
        Box<
            dyn futures::Future<Output = Result<Option<PgRow>, sqlx::Error>>
                + std::marker::Send
                + 'e,
        >,
    >
    where
        'q: 'e,
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        <&sqlx::Pool<sqlx::Postgres> as Executor<'c>>::fetch_optional(&self.0, query)
    }
    fn prepare_with<'e, 'q>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as sqlx::Database>::TypeInfo],
    ) -> Pin<
        Box<
            dyn futures::Future<Output = Result<PgStatement<'q>, sqlx::Error>>
                + std::marker::Send
                + 'e,
        >,
    >
    where
        'q: 'e,
        'c: 'e,
    {
        <&sqlx::Pool<sqlx::Postgres> as Executor<'c>>::prepare_with(&self.0, sql, parameters)
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> futures::future::BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
    {
        <&sqlx::Pool<sqlx::Postgres> as Executor<'c>>::describe(&self.0, sql)
    }
}

pub struct QueryResult {
    rows_affected: u64,
}

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
