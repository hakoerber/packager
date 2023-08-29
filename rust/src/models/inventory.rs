use super::Error;
use crate::Context;

use futures::{TryFutureExt, TryStreamExt};
use uuid::Uuid;

use tracing::Instrument;

pub struct Inventory {
    pub categories: Vec<Category>,
}

impl Inventory {
    //#[tracing::instrument]
    pub async fn load(ctx: &Context, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Self, Error> {
        let user_id = ctx.user.id.to_string();
        let categories = async {
            let mut categories = sqlx::query_as!(
                DbCategoryRow,
                "SELECT 
                id,
                name,
                description 
            FROM inventory_items_categories 
            WHERE user_id = ?",
                user_id,
            )
            .fetch(pool)
            .map_ok(|row: DbCategoryRow| row.try_into())
            .try_collect::<Vec<Result<Category, Error>>>()
            .await?
            .into_iter()
            .collect::<Result<Vec<Category>, Error>>()?;

            for category in &mut categories {
                category.populate_items(ctx, pool).await?;
            }

            Ok::<_, Error>(categories)
        }
        // .instrument(tracing::info_span!("packager::query", "query"))
        .await?;

        Ok(Self { categories })
    }
}

#[derive(Debug)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub items: Option<Vec<Item>>,
}

pub struct DbCategoryRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

impl TryFrom<DbCategoryRow> for Category {
    type Error = Error;

    fn try_from(row: DbCategoryRow) -> Result<Self, Self::Error> {
        Ok(Category {
            id: Uuid::try_parse(&row.id)?,
            name: row.name,
            description: row.description,
            items: None,
        })
    }
}

impl Category {
    //#[tracing::instrument]
    pub async fn _find(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
    ) -> Result<Option<Category>, Error> {
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        sqlx::query_as!(
            DbCategoryRow,
            "SELECT
                id,
                name,
                description
            FROM inventory_items_categories AS category
            WHERE 
                category.id = ?
                AND category.user_id = ?",
            id_param,
            user_id,
        )
        .fetch_optional(pool)
        .await?
        .map(|row| row.try_into())
        .transpose()
    }

    //#[tracing::instrument]
    pub async fn save(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        sqlx::query!(
            "INSERT INTO inventory_items_categories
                (id, name, user_id)
            VALUES
                (?, ?, ?)",
            id_param,
            name,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(id)
    }

    //#[tracing::instrument]
    pub fn items(&self) -> &Vec<Item> {
        self.items
            .as_ref()
            .expect("you need to call populate_items()")
    }

    //#[tracing::instrument]
    pub fn total_weight(&self) -> i64 {
        self.items().iter().map(|item| item.weight).sum()
    }

    //#[tracing::instrument]
    pub async fn populate_items(
        &mut self,
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
        let id = self.id.to_string();
        let user_id = ctx.user.id.to_string();
        let items = sqlx::query_as!(
            DbInventoryItemsRow,
            "SELECT
                id,
                name,
                weight,
                description,
                category_id
            FROM inventory_items
            WHERE 
                category_id = ?
                AND user_id = ?",
            id,
            user_id,
        )
        .fetch(pool)
        .map_ok(|row| row.try_into())
        .try_collect::<Vec<Result<Item, Error>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<Item>, Error>>()?;

        self.items = Some(items);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug)]
pub struct InventoryItem {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub weight: i64,
    pub category: Category,
    pub product: Option<Product>,
}

struct DbInventoryItemRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub weight: i64,
    pub category_id: String,
    pub category_name: String,
    pub category_description: Option<String>,
    pub product_id: Option<String>,
    pub product_name: Option<String>,
    pub product_description: Option<String>,
    pub product_comment: Option<String>,
}

impl TryFrom<DbInventoryItemRow> for InventoryItem {
    type Error = Error;

    fn try_from(row: DbInventoryItemRow) -> Result<Self, Self::Error> {
        Ok(InventoryItem {
            id: Uuid::try_parse(&row.id)?,
            name: row.name,
            description: row.description,
            weight: row.weight,
            category: Category {
                id: Uuid::try_parse(&row.category_id)?,
                name: row.category_name,
                description: row.category_description,
                items: None,
            },
            product: row
                .product_id
                .map(|id| -> Result<Product, Error> {
                    Ok(Product {
                        id: Uuid::try_parse(&id)?,
                        name: row.product_name.unwrap(),
                        description: row.product_description,
                        comment: row.product_comment,
                    })
                })
                .transpose()?,
        })
    }
}

impl InventoryItem {
    //#[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();

        sqlx::query_as!(
            DbInventoryItemRow,
            "SELECT
                    item.id AS id,
                    item.name AS name,
                    item.description AS description,
                    weight,
                    category.id AS category_id,
                    category.name AS category_name,
                    category.description AS category_description,
                    product.id AS product_id,
                    product.name AS product_name,
                    product.description AS product_description,
                    product.comment AS product_comment
                FROM inventory_items AS item
                INNER JOIN inventory_items_categories as category
                    ON item.category_id = category.id
                LEFT JOIN inventory_products AS product
                    ON item.product_id = product.id
                WHERE 
                    item.id = ?
                    AND item.user_id = ?",
            id_param,
            user_id,
        )
        .fetch_optional(pool)
        .await?
        .map(|row| row.try_into())
        .transpose()
    }

    //#[tracing::instrument]
    pub async fn name_exists(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
    ) -> Result<bool, Error> {
        let user_id = ctx.user.id.to_string();
        Ok(sqlx::query!(
            "SELECT id
            FROM inventory_items
            WHERE 
                name = ?
                AND user_id = ?",
            name,
            user_id
        )
        .fetch_optional(pool)
        .await?
        .map(|_row| ())
        .is_some())
    }

    //#[tracing::instrument]
    pub async fn delete(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
    ) -> Result<bool, Error> {
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        let results = sqlx::query!(
            "DELETE FROM inventory_items
            WHERE 
                id = ?
                AND user_id = ?",
            id_param,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(results.rows_affected() != 0)
    }

    //#[tracing::instrument]
    pub async fn update(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
        name: &str,
        weight: u32,
    ) -> Result<Uuid, Error> {
        let user_id = ctx.user.id.to_string();
        let weight = i64::try_from(weight).unwrap();

        let id_param = id.to_string();
        Ok(sqlx::query!(
            "UPDATE inventory_items AS item
            SET
                name = ?,
                weight = ?
            WHERE 
                item.id = ?
                AND item.user_id = ?
            RETURNING inventory_items.category_id AS id
            ",
            name,
            weight,
            id_param,
            user_id,
        )
        .fetch_one(pool)
        .map_ok(|row| Uuid::try_parse(&row.id))
        .await??)
    }

    //#[tracing::instrument]
    pub async fn save(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
        category_id: Uuid,
        weight: u32,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        let category_id_param = category_id.to_string();

        sqlx::query!(
            "INSERT INTO inventory_items
                (id, name, description, weight, category_id, user_id)
            VALUES
                (?, ?, ?, ?, ?, ?)",
            id_param,
            name,
            "",
            weight,
            category_id_param,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(id)
    }

    //#[tracing::instrument]
    pub async fn get_category_max_weight(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        category_id: Uuid,
    ) -> Result<i64, Error> {
        let user_id = ctx.user.id.to_string();
        let category_id_param = category_id.to_string();
        let weight = sqlx::query!(
            "
                SELECT COALESCE(MAX(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                WHERE 
                    category_id = ?
                    AND category.user_id = ?
            ",
            category_id_param,
            user_id,
        )
        .fetch_one(pool)
        .map_ok(|row| {
            // convert to i64 because that the default integer type, but looks
            // like COALESCE return i32?
            i64::from(row.weight)
        })
        .await?;

        Ok(weight)
    }
}

#[derive(Debug)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub weight: i64,
    pub category_id: Uuid,
}

pub struct DbInventoryItemsRow {
    pub id: String,
    pub name: String,
    pub weight: i64,
    pub description: Option<String>,
    pub category_id: String,
}

impl TryFrom<DbInventoryItemsRow> for Item {
    type Error = Error;

    fn try_from(row: DbInventoryItemsRow) -> Result<Self, Self::Error> {
        Ok(Item {
            id: Uuid::try_parse(&row.id)?,
            name: row.name,
            description: row.description, // TODO
            weight: row.weight,
            category_id: Uuid::try_parse(&row.category_id)?,
        })
    }
}

impl Item {
    //#[tracing::instrument]
    pub async fn _get_category_total_picked_weight(
        ctx: &Context,
        pool: &sqlx::Pool<sqlx::Sqlite>,
        category_id: Uuid,
    ) -> Result<i64, Error> {
        let user_id = ctx.user.id.to_string();
        let category_id_param = category_id.to_string();
        Ok(sqlx::query!(
            "
                SELECT COALESCE(SUM(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                INNER JOIN trips_items as t_item
                    ON i_item.id = t_item.item_id
                WHERE 
                    category_id = ?
                    AND category.user_id = ?
                    AND t_item.pick = 1
            ",
            category_id_param,
            user_id,
        )
        .fetch_one(pool)
        .map_ok(|row| {
            // convert to i64 because that the default integer type, but looks
            // like COALESCE return i32?
            i64::from(row.weight)
        })
        .await?)
    }
}
