pub mod inventory;
pub mod products;
pub mod trips;

pub mod crud {
    use async_trait::async_trait;

    use crate::{Context, error::Error};

    #[async_trait]
    pub trait Create: Sized {
        type Id: Sized + Send + Sync + 'static;
        type Container: Sized + Send + Sync + 'static;
        type Info: Sized + Send + Sync + 'static;

        fn new_id() -> Self::Id;

        async fn create(
            ctx: &Context,
            pool: &database::Pool,
            container: Self::Container,
            info: Self::Info,
        ) -> Result<Self::Id, Error>;
    }

    #[async_trait]
    pub trait Read: Sized {
        type Reference;
        type Container;

        async fn findall(
            ctx: &Context,
            pool: &database::Pool,
            container: Self::Container,
        ) -> Result<Vec<Self>, Error>;

        async fn find(
            ctx: &Context,
            pool: &database::Pool,
            reference: Self::Reference,
        ) -> Result<Option<Self>, Error>;
    }

    #[async_trait]
    pub trait Update: Sized {
        type Reference;
        type UpdateElement;

        async fn update(
            ctx: &Context,
            pool: &database::Pool,
            reference: Self::Reference,
            update: Self::UpdateElement,
        ) -> Result<Option<Self>, Error>;
    }

    pub trait Container {
        type Id: Copy;
        type Reference;

        fn with_id(&self, id: Self::Id) -> Self::Reference;
    }

    #[async_trait]
    pub trait Delete: Sized {
        type Id: Send + Copy;
        type Container: Send + Sync + Container<Reference = Self::Reference, Id = Self::Id>;
        type Reference: Send + Sync;

        async fn delete<'c, T>(
            ctx: &Context,
            db: T,
            reference: &Self::Reference,
        ) -> Result<bool, Error>
        where
            // we require something that allows us to get something that implements
            // executor from an SQLx database
            //
            // in practice, this will either be a pool or a transaction
            //
            // * A pool will let us begin() a new transaction directly and then
            //   acquire() a new conncetion
            //
            // * A transaction will begin() (a noop) and then acquire() a new connection
            T: sqlx::Acquire<'c, Database = sqlx::Postgres> + Send + std::fmt::Debug;

        #[allow(dead_code)]
        async fn delete_all<'c>(
            ctx: &Context,
            pool: &'c database::Pool,
            container: Self::Container,
            ids: Vec<Self::Id>,
        ) -> Result<bool, Error> {
            use sqlx::Acquire as _;

            let mut transaction = pool.begin().await?;
            let conn = transaction.acquire().await?;

            for id in ids {
                if !Self::delete(ctx, &mut *conn, &container.with_id(id)).await? {
                    // transaction will rollback on drop
                    return Ok(false);
                }
            }

            transaction.commit().await?;

            Ok(true)
        }
    }

    #[async_trait]
    pub trait Toggle: Sized {
        type Reference: Sized + Send + Sync + 'static;

        async fn set(
            ctx: &Context,
            pool: &database::Pool,
            reference: Self::Reference,
            value: bool,
        ) -> Result<(), crate::Error>;
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

    use crate::{AppState, models::user::User};
    use axum::{
        Extension, Form,
        body::Body,
        extract::{Path, Query, State},
        http::HeaderMap,
        response::Response,
    };

    #[async_trait]
    pub trait Create: super::crud::Create {
        type Form: Send + Sync + 'static;

        type UrlParams: Send + Sync + 'static;

        // const URL: &'static str;

        async fn create(
            user: Extension<User>,
            state: State<AppState>,
            headers: HeaderMap,
            path: Path<Self::UrlParams>,
            form: Form<Self::Form>,
        ) -> Result<Response<Body>, crate::Error>;
    }

    #[async_trait]
    #[allow(dead_code)]
    pub trait Read: super::crud::Read {
        type UrlParams: Send + Sync + 'static;
        type QueryParams: Send + Sync + 'static;

        const URL: &'static str;

        async fn read(
            user: Extension<User>,
            state: State<AppState>,
            headers: HeaderMap,
            query: Query<Self::QueryParams>,
            path: Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error>;
    }

    #[async_trait]
    #[allow(dead_code)]
    pub trait Update: super::crud::Update {
        type UrlParams: Send + Sync + 'static;
        type UpdateForm: Send + Sync + 'static;

        const URL: &'static str;

        async fn start(
            user: Extension<User>,
            state: State<AppState>,
            headers: HeaderMap,
            path: Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error>;

        async fn save(
            user: Extension<User>,
            state: State<AppState>,
            headers: HeaderMap,
            path: Path<Self::UrlParams>,
            form: Form<Self::UpdateForm>,
        ) -> Result<Response<Body>, crate::Error>;

        async fn cancel(
            user: Extension<User>,
            state: State<AppState>,
            headers: HeaderMap,
            path: Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error>;
    }

    #[async_trait]
    pub trait ToggleFallback: super::crud::Toggle {
        type UrlParams: Send + Sync + 'static;

        const URL_TRUE: &'static str;
        const URL_FALSE: &'static str;

        async fn set(
            current_user: User,
            state: AppState,
            headers: HeaderMap,
            params: Self::UrlParams,
            value: bool,
        ) -> Result<Response<Body>, crate::Error>;

        async fn set_true(
            Extension(user): Extension<User>,
            State(state): State<AppState>,
            headers: HeaderMap,
            Path(path): Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error> {
            <Self as ToggleFallback>::set(user, state, headers, path, true).await
        }

        async fn set_false(
            Extension(user): Extension<User>,
            State(state): State<AppState>,
            headers: HeaderMap,
            Path(path): Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error> {
            <Self as ToggleFallback>::set(user, state, headers, path, false).await
        }

        fn router() -> axum::Router<AppState>;
    }

    #[async_trait]
    pub trait ToggleHtmx: super::crud::Toggle {
        type UrlParams: Send + Sync + 'static;

        const URL_TRUE: &'static str;
        const URL_FALSE: &'static str;

        async fn set(
            current_user: User,
            state: AppState,
            params: Self::UrlParams,
            value: bool,
        ) -> Result<(crate::Context, AppState, Self::UrlParams), crate::Error>;

        async fn response(
            ctx: &crate::Context,
            state: AppState,
            params: Self::UrlParams,
        ) -> Result<Response<Body>, crate::Error>;

        async fn on(
            Extension(user): Extension<User>,
            State(state): State<AppState>,
            Path(path): Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error> {
            let (ctx, state, params) = <Self as ToggleHtmx>::set(user, state, path, true).await?;
            <Self as ToggleHtmx>::response(&ctx, state, params).await
        }

        async fn off(
            Extension(user): Extension<User>,
            State(state): State<AppState>,
            Path(path): Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error> {
            let (ctx, state, params) = <Self as ToggleHtmx>::set(user, state, path, false).await?;
            <Self as ToggleHtmx>::response(&ctx, state, params).await
        }

        fn router() -> axum::Router<AppState>;
    }

    pub trait Toggle: ToggleHtmx + ToggleFallback {
        fn router() -> axum::Router<AppState> {
            axum::Router::new()
                .merge(<Self as ToggleHtmx>::router())
                .merge(<Self as ToggleFallback>::router())
        }
    }

    #[async_trait]
    pub trait Delete: super::crud::Delete {
        type UrlParams: Send + Sync + 'static;

        // const URL: &'static str;

        async fn delete(
            user: Extension<User>,
            state: State<AppState>,
            headers: HeaderMap,
            path: Path<Self::UrlParams>,
        ) -> Result<Response<Body>, crate::Error>;
    }

    pub trait Router: Create + Delete {
        fn router() -> axum::Router<AppState>;
    }
}
