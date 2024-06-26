use super::Error;
use crate::{sqlite, Context};

use uuid::Uuid;

pub struct Inventory {
    pub categories: Vec<Category>,
}

impl Inventory {
    #[tracing::instrument]
    pub async fn load(ctx: &Context, pool: &sqlite::Pool) -> Result<Self, Error> {
        let user_id = ctx.user.id.to_string();

        let mut categories = crate::query_all!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Inventory,
            },
            pool,
            DbCategoryRow,
            Category,
            "SELECT
                    id,
                    name,
                    description 
                FROM inventory_items_categories 
                WHERE user_id = ?",
            user_id
        )
        .await?;

        for category in &mut categories {
            category.populate_items(ctx, pool).await?;
        }

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
    #[tracing::instrument]
    pub async fn _find(
        ctx: &Context,
        pool: &sqlite::Pool,
        id: Uuid,
    ) -> Result<Option<Category>, Error> {
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        crate::query_one!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Inventory,
            },
            pool,
            DbCategoryRow,
            Category,
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
        .await
    }

    #[tracing::instrument]
    pub async fn save(ctx: &Context, pool: &sqlite::Pool, name: &str) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Insert,
                component: sqlite::Component::Inventory,
            },
            pool,
            "INSERT INTO inventory_items_categories
                (id, name, user_id)
            VALUES
                (?, ?, ?)",
            id_param,
            name,
            user_id,
        )
        .await?;

        Ok(id)
    }

    #[tracing::instrument]
    pub fn items(&self) -> &Vec<Item> {
        self.items
            .as_ref()
            .expect("you need to call populate_items()")
    }

    #[tracing::instrument]
    pub fn total_weight(&self) -> i64 {
        self.items().iter().map(|item| item.weight).sum()
    }

    #[tracing::instrument]
    pub async fn populate_items(
        &mut self,
        ctx: &Context,
        pool: &sqlite::Pool,
    ) -> Result<(), Error> {
        let id = self.id.to_string();
        let user_id = ctx.user.id.to_string();
        let items = crate::query_all!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Inventory,
            },
            pool,
            DbInventoryItemsRow,
            Item,
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
        .await?;

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
    #[tracing::instrument]
    pub async fn find(ctx: &Context, pool: &sqlite::Pool, id: Uuid) -> Result<Option<Self>, Error> {
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();

        crate::query_one!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Inventory,
            },
            pool,
            DbInventoryItemRow,
            Self,
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
        .await
    }

    #[tracing::instrument]
    pub async fn name_exists(
        ctx: &Context,
        pool: &sqlite::Pool,
        name: &str,
    ) -> Result<bool, Error> {
        let user_id = ctx.user.id.to_string();
        crate::query_exists!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Inventory,
            },
            pool,
            "SELECT id
            FROM inventory_items
            WHERE 
                name = ?
                AND user_id = ?",
            name,
            user_id
        )
        .await
    }

    #[tracing::instrument]
    pub async fn delete(ctx: &Context, pool: &sqlite::Pool, id: Uuid) -> Result<bool, Error> {
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        let results = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Delete,
                component: sqlite::Component::Inventory,
            },
            pool,
            "DELETE FROM inventory_items
            WHERE 
                id = ?
                AND user_id = ?",
            id_param,
            user_id,
        )
        .await?;

        Ok(results.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn update(
        ctx: &Context,
        pool: &sqlite::Pool,
        id: Uuid,
        name: &str,
        weight: u32,
    ) -> Result<Uuid, Error> {
        let user_id = ctx.user.id.to_string();
        let weight = i64::from(weight);

        let id_param = id.to_string();
        crate::execute_returning_uuid!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Update,
                component: sqlite::Component::Inventory,
            },
            pool,
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
        .await
    }

    #[tracing::instrument]
    pub async fn save(
        ctx: &Context,
        pool: &sqlite::Pool,
        name: &str,
        category_id: Uuid,
        weight: u32,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let user_id = ctx.user.id.to_string();
        let category_id_param = category_id.to_string();

        crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Insert,
                component: sqlite::Component::Inventory,
            },
            pool,
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
        .await?;

        Ok(id)
    }

    #[tracing::instrument]
    pub async fn get_category_max_weight(
        ctx: &Context,
        pool: &sqlite::Pool,
        category_id: Uuid,
    ) -> Result<i64, Error> {
        let user_id = ctx.user.id.to_string();
        let category_id_param = category_id.to_string();
        let weight = crate::execute_returning!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Inventory,
            },
            pool,
            "
                SELECT COALESCE(MAX(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                WHERE 
                    category_id = ?
                    AND category.user_id = ?
            ",
            i64,
            |row| i64::from(row.weight),
            category_id_param,
            user_id,
        )
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
    #[tracing::instrument]
    pub async fn _get_category_total_picked_weight(
        ctx: &Context,
        pool: &sqlite::Pool,
        category_id: Uuid,
    ) -> Result<i64, Error> {
        let user_id = ctx.user.id.to_string();
        let category_id_param = category_id.to_string();
        crate::execute_returning!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Inventory,
            },
            pool,
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
            i64,
            |row| i64::from(row.weight),
            category_id_param,
            user_id,
        )
        .await
    }
}
