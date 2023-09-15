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
        type Id;
        type Filter;

        async fn delete(
            ctx: &Context,
            pool: impl sqlx::Acquire,
            filter: Self::Filter,
            id: Self::Id,
        ) -> Result<bool, Error>;

        async fn delete_all(
            ctx: &Context,
            pool: &sqlite::Pool,
            filter: Self::Filter,
            ids: Vec<Self::Id>,
        ) -> Result<bool, Error> {
            let mut transaction = pool.begin().await?;

            for id in ids {
                Self::delete(ctx, &mut transaction, filter, id).await?;
            }
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
