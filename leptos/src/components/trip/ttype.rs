use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TType {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TripTypeAttribute {
    #[serde(rename = "name")]
    Name,
}

#[derive(Debug)]
pub struct TripsType {
    pub id: Uuid,
    pub name: String,
}
#[cfg(feature = "ssr")]
pub mod model {
    use super::*;
    use crate::{components::Component, context::Context, error::RunError};

    pub struct TripTypeRow {
        pub id: Uuid,
        pub name: String,
        pub active: bool,
    }

    impl TryFrom<TripTypeRow> for TType {
        type Error = RunError;

        fn try_from(row: TripTypeRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: row.id,
                name: row.name,
                active: row.active,
            })
        }
    }

    impl TripsType {
        #[tracing::instrument]
        pub async fn all(ctx: &Context, pool: &database::Pool) -> Result<Vec<Self>, RunError> {
            database::query_all!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: Component::Trip,
                },
                pool,
                DbTripsTypesRow,
                Self,
                RunError,
                "SELECT
                id,
                name
            FROM trip_types
            WHERE user_id = $1",
                ctx.user.id
            )
            .await
        }

        #[tracing::instrument]
        pub async fn save(
            ctx: &Context,
            pool: &database::Pool,
            name: &str,
        ) -> Result<Uuid, RunError> {
            let id = Uuid::new_v4();
            database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Insert,
                    component: Component::Trip,
                },
                pool,
                RunError,
                "INSERT INTO trip_types
                (id, name, user_id)
            VALUES
                ($1, $2, $3)",
                id,
                name,
                ctx.user.id
            )
            .await?;

            Ok(id)
        }

        #[tracing::instrument]
        pub async fn set_name(
            ctx: &Context,
            pool: &database::Pool,
            id: Uuid,
            new_name: &str,
        ) -> Result<bool, RunError> {
            let result = database::execute!(
                &database::QueryClassification {
                    query_type: database::QueryType::Update,
                    component: Component::Trip,
                },
                pool,
                RunError,
                "UPDATE trip_types
            SET name = $1
            WHERE id = $2 and user_id = $3",
                new_name,
                id,
                ctx.user.id
            )
            .await?;

            Ok(result.rows_affected() != 0)
        }
    }

    pub struct DbTripsTypesRow {
        pub id: Uuid,
        pub name: String,
    }

    impl TryFrom<DbTripsTypesRow> for TripsType {
        type Error = RunError;

        fn try_from(row: DbTripsTypesRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: row.id,
                name: row.name,
            })
        }
    }
}
