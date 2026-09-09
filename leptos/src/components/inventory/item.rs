use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod trip {
    #[derive(Debug)]
    pub struct Trip {
        pub name: String,
        // pub date: crate::domains::trips::TripDate,
        pub state: super::super::super::trip::TripState,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub weight: i32,
    pub category_id: Uuid,
}

#[cfg(feature = "ssr")]
pub mod model {
    use super::*;
    use crate::{components::Component, context::Context, error::RunError};

    pub struct DbInventoryItemsRow {
        pub id: Uuid,
        pub name: String,
        pub weight: i32,
        pub description: Option<String>,
        pub category_id: Uuid,
    }

    impl TryFrom<DbInventoryItemsRow> for Item {
        type Error = RunError;

        fn try_from(row: DbInventoryItemsRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: row.id,
                name: row.name,
                description: row.description, // TODO
                weight: row.weight,
                category_id: row.category_id,
            })
        }
    }

    impl Item {
        #[tracing::instrument(skip(pool))]
        pub async fn _get_category_total_picked_weight(
            ctx: &Context,
            pool: &database::Pool,
            category_id: Uuid,
        ) -> Result<i32, RunError> {
            database::execute_returning!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: Component::Inventory,
                },
                pool,
                RunError,
                "
                SELECT COALESCE(SUM(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                INNER JOIN trip_items as t_item
                    ON i_item.id = t_item.item_id
                WHERE
                    category_id = $1
                    AND category.user_id = $2
                    AND t_item.pick = true
            ",
                i32,
                |row| i32::try_from(row.weight.unwrap()).unwrap(),
                category_id,
                ctx.user.id,
            )
            .await
        }
    }
}
