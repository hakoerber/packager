use sqlx::{ConnectOptions as _, PgPool, postgres::PgConnectOptions};
use url::Url;

use super::error::InitError;

use tracing::Instrument as _;

pub struct DB;

impl DB {
    fn opts(url: &str) -> Result<PgConnectOptions, InitError> {
        Ok(PgConnectOptions::from_url(&Url::parse(url).map_err(
            |err| <_ as Into<InitError>>::into((url.to_owned(), err)),
        )?)?)
    }
}

impl super::Database for DB {
    type Pool = sqlx::Pool<sqlx::Postgres>;

    #[tracing::instrument]
    async fn init_database_pool(url: &str) -> Result<Self::Pool, InitError> {
        async {
            let options = Self::opts(url)?;

            PgPool::connect_with(options)
                .await
                .map_err(Into::<InitError>::into)
        }
        .instrument(tracing::info_span!("packager::sql::pool"))
        .await
    }

    #[tracing::instrument]
    async fn migrate(url: &str) -> Result<(), InitError> {
        async {
            let pool = PgPool::connect_with(Self::opts(url)?).await?;

            sqlx::migrate!("../migrations")
                .run(&pool)
                .await
                .map_err(Into::<InitError>::into)
        }
        .instrument(tracing::info_span!("packager::sql::migrate"))
        .await?;

        Ok(())
    }
}
