use std::pin::Pin;

use futures::Stream;
use sqlx::{
    Acquire, Execute, Executor, Transaction,
    pool::PoolConnection,
    postgres::{PgQueryResult, PgRow, PgStatement},
};

#[derive(Clone, Debug)]
pub struct Pool(pub(crate) sqlx::Pool<sqlx::Postgres>);

impl Pool {
    pub async fn begin(&self) -> Result<Transaction<'static, sqlx::Postgres>, sqlx::Error> {
        self.0.begin().await
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
