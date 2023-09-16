pub mod trips;

pub mod crud {
    use async_trait::async_trait;

    use crate::{models::Error, sqlite, Context};

    #[async_trait]
    pub trait Create: Sized {
        type Id;
        type Filter;
        type Info;

        async fn create(
            ctx: &Context,
            pool: &sqlite::Pool,
            filter: Self::Filter,
            info: Self::Info,
        ) -> Result<Self::Id, Error>;
    }

    #[async_trait]
    pub trait Read: Sized {
        type Filter;
        type Id;

        async fn findall(
            ctx: &Context,
            pool: &sqlite::Pool,
            filter: Self::Filter,
        ) -> Result<Vec<Self>, Error>;

        async fn find(
            ctx: &Context,
            pool: &sqlite::Pool,
            filter: Self::Filter,
            id: Self::Id,
        ) -> Result<Option<Self>, Error>;
    }

    #[async_trait]
    pub trait Update: Sized {
        type Id;
        type Filter;
        type Update;

        async fn update(
            ctx: &Context,
            pool: &sqlite::Pool,
            filter: Self::Filter,
            id: Self::Id,
            update: Self::Update,
        ) -> Result<Option<Self>, Error>;
    }

    #[async_trait]
    pub trait Delete: Sized {
        type Id: Send + Copy;
        type Filter: Send + Sync;

        async fn delete<'c, T>(
            ctx: &Context,
            db: T,
            filter: &Self::Filter,
            id: Self::Id,
        ) -> Result<bool, Error>
        where
            // we require something that allows us to get something that implements
            // executor from a Sqlite database
            //
            // in practice, this will either be a pool or a transaction
            //
            // * A pool will let us begin() a new transaction directly and then
            //   acquire() a new conncetion
            //
            // * A transaction will begin() (a noop) and then acquire() a new connection
            T: sqlx::Acquire<'c, Database = sqlx::Sqlite> + Send + std::fmt::Debug;

        async fn delete_all<'c>(
            ctx: &Context,
            pool: &'c sqlite::Pool,
            filter: Self::Filter,
            ids: Vec<Self::Id>,
        ) -> Result<bool, Error> {
            use sqlx::Acquire as _;

            let mut transaction = pool.begin().await?;
            let conn = transaction.acquire().await?;

            for id in ids {
                if !Self::delete(ctx, &mut *conn, &filter, id).await? {
                    // transaction will rollback on drop
                    return Ok(false);
                }
            }

            transaction.commit().await?;

            Ok(true)
        }
    }
}

pub mod view {
    use maud::Markup;

    pub trait View {
        type Input;

        fn build(&self, input: Self::Input) -> Markup;
    }
}

pub mod route {
    use async_trait::async_trait;

    use crate::AppState;
    use axum::{
        body::BoxBody,
        extract::{Path, Query, State},
        http::HeaderMap,
        response::Response,
        Extension, Form,
    };

    pub enum Method {
        Get,
        Post,
    }

    #[async_trait]
    pub trait Create: super::crud::Create {
        type Form: Send + Sync + 'static;

        type ParentUrlParams: Send + Sync + 'static;
        type UrlParams: Send + Sync + 'static;

        const URL: &'static str;

        async fn create(
            user: Extension<crate::models::user::User>,
            state: State<AppState>,
            headers: HeaderMap,
            path: Path<(Self::ParentUrlParams, Self::UrlParams)>,
            form: Form<Self::Form>,
        ) -> Result<Response<BoxBody>, crate::Error>;

        fn with_prefix(prefix: &'static str) -> String {
            format!("{}{}", prefix, Self::URL)
        }
    }

    #[async_trait]
    pub trait Read: super::crud::Read {
        type UrlParams: Send + Sync + 'static;
        type QueryParams: Send + Sync + 'static;

        const URL: &'static str;

        async fn read(
            user: Extension<crate::models::user::User>,
            state: State<AppState>,
            headers: HeaderMap,
            query: Query<Self::QueryParams>,
            path: Path<Self::UrlParams>,
        ) -> Result<Response<BoxBody>, crate::Error>;

        fn with_prefix(prefix: &'static str) -> String {
            format!("{}{}", prefix, Self::URL)
        }
    }

    #[async_trait]
    pub trait Delete: super::crud::Delete {
        type ParentUrlParams: Send + Sync + 'static;
        type UrlParams: Send + Sync + 'static;

        const URL: &'static str;

        async fn delete(
            user: Extension<crate::models::user::User>,
            state: State<AppState>,
            headers: HeaderMap,
            path: Path<(Self::ParentUrlParams, Self::UrlParams)>,
        ) -> Result<Response<BoxBody>, crate::Error>;

        fn with_prefix(prefix: &'static str) -> String {
            format!("{}{}", prefix, Self::URL)
        }
    }
}
