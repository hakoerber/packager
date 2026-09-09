use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub items: Option<Vec<super::item::Item>>,
}

#[cfg(feature = "ssr")]
pub mod model {
    use super::*;
    use crate::{components::Component, context::Context, error::RunError};

    pub struct DbCategoryRow {
        pub id: Uuid,
        pub name: String,
    }

    impl TryFrom<DbCategoryRow> for Category {
        type Error = RunError;

        fn try_from(row: DbCategoryRow) -> Result<Self, Self::Error> {
            Ok(Self {
                id: row.id,
                name: row.name,
                items: None,
            })
        }
    }

    impl Category {
        #[tracing::instrument]
        pub async fn _find(
            ctx: &Context,
            pool: &database::Pool,
            id: Uuid,
        ) -> Result<Option<Self>, RunError> {
            database::query_one!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: Component::Inventory,
                },
                pool,
                DbCategoryRow,
                Category,
                RunError,
                "SELECT
                id,
                name
            FROM inventory_items_categories AS category
            WHERE
                category.id = $1
                AND category.user_id = $2",
                id,
                ctx.user.id,
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
                    component: Component::Inventory,
                },
                pool,
                RunError,
                "INSERT INTO inventory_items_categories
                (id, name, user_id)
            VALUES
                ($1, $2, $3)",
                id,
                name,
                ctx.user.id,
            )
            .await?;

            Ok(id)
        }

        #[tracing::instrument]
        pub fn items(&self) -> &Vec<super::super::item::Item> {
            self.items
                .as_ref()
                .expect("you need to call populate_items()")
        }

        #[tracing::instrument]
        pub fn total_weight(&self) -> i32 {
            self.items().iter().map(|item| item.weight).sum()
        }

        #[tracing::instrument]
        pub async fn populate_items(
            &mut self,
            ctx: &Context,
            pool: &database::Pool,
        ) -> Result<(), RunError> {
            let items = database::query_all!(
                &database::QueryClassification {
                    query_type: database::QueryType::Select,
                    component: Component::Inventory,
                },
                pool,
                super::super::item::model::DbInventoryItemsRow,
                super::super::item::Item,
                RunError,
                "SELECT
                id,
                name,
                weight,
                description,
                category_id
            FROM inventory_items
            WHERE
                category_id = $1
                AND user_id = $2",
                self.id,
                ctx.user.id,
            )
            .await?;

            self.items = Some(items);
            Ok(())
        }
    }
}
