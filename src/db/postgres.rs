use sqlx::{postgres::PgConnectOptions, PgPool};

use super::StartError;

use tracing::Instrument as _;

pub(crate) struct DB;

impl DB {
    fn opts(url: &str) -> Result<PgConnectOptions, StartError> {
        url.parse().map_err(Into::into)
    }
}

impl super::Database for DB {
    type Pool = sqlx::Pool<sqlx::Postgres>;

    #[tracing::instrument]
    async fn init_database_pool(url: &str) -> Result<Self::Pool, StartError> {
        async {
            let options = Self::opts(url)?;

            PgPool::connect_with(options)
                .await
                .map_err(Into::<StartError>::into)
        }
        .instrument(tracing::info_span!("packager::sql::pool"))
        .await
    }

    #[tracing::instrument]
    async fn migrate(url: &str) -> Result<(), StartError> {
        async {
            let pool = PgPool::connect_with(Self::opts(url)?).await?;

            sqlx::migrate!()
                .run(&pool)
                .await
                .map_err(Into::<StartError>::into)
        }
        .instrument(tracing::info_span!("packager::sql::migrate"))
        .await?;

        Ok(())
    }
}
