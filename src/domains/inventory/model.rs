use crate::Context;
use crate::error::Error;

use uuid::Uuid;

#[derive(Debug)]
pub struct Product {
    #[allow(dead_code)]
    pub id: Uuid,
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
}

pub struct Inventory {
    pub categories: Vec<Category>,
}

impl Inventory {
    #[tracing::instrument]
    pub async fn load(ctx: &Context, pool: &database::Pool) -> Result<Self, Error> {
        let mut categories = database::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Inventory,
            },
            pool,
            DbCategoryRow,
            Category,
            "SELECT
                    id,
                    name
                FROM inventory_items_categories
                WHERE user_id = $1",
            ctx.user.id
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
    pub items: Option<Vec<Item>>,
}

pub struct DbCategoryRow {
    pub id: Uuid,
    pub name: String,
}

impl TryFrom<DbCategoryRow> for Category {
    type Error = Error;

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
    pub async fn _find(ctx: &Context, pool: &db::Pool, id: Uuid) -> Result<Option<Self>, Error> {
        crate::query_one!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Inventory,
            },
            pool,
            DbCategoryRow,
            Category,
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
    pub async fn save(ctx: &Context, pool: &db::Pool, name: &str) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Insert,
                component: db::Component::Inventory,
            },
            pool,
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
    pub fn items(&self) -> &Vec<Item> {
        self.items
            .as_ref()
            .expect("you need to call populate_items()")
    }

    #[tracing::instrument]
    pub fn total_weight(&self) -> i32 {
        self.items().iter().map(|item| item.weight).sum()
    }

    #[tracing::instrument]
    pub async fn populate_items(&mut self, ctx: &Context, pool: &db::Pool) -> Result<(), Error> {
        let items = crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Inventory,
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

#[derive(Debug)]
pub struct InventoryItemTrip {
    pub name: String,
    // pub date: crate::domains::trips::TripDate,
    pub state: crate::domains::trips::TripState,
}

#[derive(Debug)]
struct DbInventoryItemRows {
    first: DbInventoryItemRow,
    rest: Vec<DbInventoryItemRow>,
}

impl DbInventoryItemRows {
    fn first(&self) -> &DbInventoryItemRow {
        &self.first
    }
}

impl<'a> DbInventoryItemRows {
    #[allow(dead_code)]
    fn iter(&'a self) -> DbInventoryItemRowsIterRef<'a> {
        DbInventoryItemRowsIterRef {
            first: Some(&self.first),
            inner_iter: self.rest.iter(),
        }
    }

    fn iter_mut(&'a mut self) -> DbInventoryItemRowsIterRefMut<'a> {
        DbInventoryItemRowsIterRefMut {
            first: Some(&mut self.first),
            inner_iter: self.rest.iter_mut(),
        }
    }
}

#[allow(dead_code)]
struct DbInventoryItemRowsIterRef<'a> {
    first: Option<&'a DbInventoryItemRow>,
    inner_iter: std::slice::Iter<'a, DbInventoryItemRow>,
}

impl<'a> Iterator for DbInventoryItemRowsIterRef<'a> {
    type Item = &'a DbInventoryItemRow;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first.take() {
            Some(first)
        } else {
            self.inner_iter.next()
        }
    }
}

struct DbInventoryItemRowsIterRefMut<'a> {
    first: Option<&'a mut DbInventoryItemRow>,
    inner_iter: std::slice::IterMut<'a, DbInventoryItemRow>,
}

impl<'a> Iterator for DbInventoryItemRowsIterRefMut<'a> {
    type Item = &'a mut DbInventoryItemRow;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first.take() {
            Some(first)
        } else {
            self.inner_iter.next()
        }
    }
}

struct DbInventoryItemRowsIter {
    first: Option<DbInventoryItemRow>,
    inner_iter: std::vec::IntoIter<DbInventoryItemRow>,
}

impl Iterator for DbInventoryItemRowsIter {
    type Item = DbInventoryItemRow;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first.take() {
            Some(first)
        } else {
            self.inner_iter.next()
        }
    }
}

impl IntoIterator for DbInventoryItemRows {
    type Item = DbInventoryItemRow;

    type IntoIter = DbInventoryItemRowsIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            first: Some(self.first),
            inner_iter: self.rest.into_iter(),
        }
    }
}

#[expect(clippy::fallible_impl_from, reason = "panics only on buggy code")]
impl From<Vec<DbInventoryItemRow>> for DbInventoryItemRows {
    fn from(mut value: Vec<DbInventoryItemRow>) -> Self {
        match value.pop() {
            Some(first) => Self { first, rest: value },
            None => panic!("received empty vec, this is a bug"),
        }
    }
}

#[derive(Debug)]
struct DbInventoryItemRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub weight: i32,
    pub category_id: Uuid,
    pub category_name: String,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub product_description: Option<String>,
    pub trip_name: Option<String>,
    // pub trip_date: Option<crate::domains::trips::TripDate>,
    pub trip_state: Option<crate::domains::trips::TripState>,
}

#[derive(Debug)]
pub struct InventoryItem {
    #[allow(dead_code)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub weight: i32,
    pub category: Category,
    pub product: Option<Product>,
    pub trips: Vec<InventoryItemTrip>,
}

impl TryFrom<DbInventoryItemRows> for InventoryItem {
    type Error = Error;

    fn try_from(mut rows: DbInventoryItemRows) -> Result<Self, Self::Error> {
        let first_id = rows.first().id;

        let mut trips: Vec<InventoryItemTrip> = vec![];

        for row in rows.iter_mut() {
            assert_eq!(row.id, first_id);
            if let Some(name) = row.trip_name.take() {
                // safe because trip_id is non-NULL
                let state = row.trip_state.take().unwrap();
                trips.push(InventoryItemTrip { name, state });
            }
        }

        let item = rows.first;

        Ok(Self {
            id: item.id,
            name: item.name,
            description: item.description,
            weight: item.weight,
            category: Category {
                id: item.category_id,
                name: item.category_name,
                items: None,
            },
            product: item
                .product_id
                .map(|id| -> Result<Product, Error> {
                    Ok(Product {
                        id,
                        name: item.product_name.unwrap(),
                        description: item.product_description,
                    })
                })
                .transpose()?,
            trips,
        })
    }
}

impl InventoryItem {
    #[tracing::instrument]
    pub async fn find(ctx: &Context, pool: &db::Pool, id: Uuid) -> Result<Option<Self>, Error> {
        crate::query_many_to_many_single!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Inventory,
            },
            pool,
            DbInventoryItemRow,
            DbInventoryItemRows,
            Self,
            r#"SELECT
                    item.id AS id,
                    item.name AS name,
                    item.description AS description,
                    weight,
                    category.id AS category_id,
                    category.name AS category_name,
                    product.id AS "product_id?",
                    product.name AS "product_name?",
                    product.description AS "product_description?",
                    trip.name AS "trip_name?",
                    -- trip.date AS "trip_date?: crate::domains::trips::TripDate",
                    trip.state AS "trip_state?: crate::domains::trips::TripState"
                FROM inventory_items AS item
                INNER JOIN inventory_items_categories as category
                    ON item.category_id = category.id
                LEFT JOIN products AS product
                    ON item.product_id = product.id
                LEFT OUTER JOIN trip_items as ti
                    ON ti.item_id = item.id
                    AND ti.pick = TRUE
                LEFT OUTER JOIN trips as trip
                    ON ti.trip_id = trip.id
                WHERE
                    item.id = $1
                    AND item.user_id = $2"#,
            id,
            ctx.user.id,
        )
        .await
    }

    #[tracing::instrument]
    pub async fn name_exists(ctx: &Context, pool: &db::Pool, name: &str) -> Result<bool, Error> {
        crate::query_exists!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Inventory,
            },
            pool,
            "SELECT id
            FROM inventory_items
            WHERE
                name = $1
                AND user_id = $2",
            name,
            ctx.user.id
        )
        .await
    }

    #[tracing::instrument]
    pub async fn delete(ctx: &Context, pool: &db::Pool, id: Uuid) -> Result<bool, Error> {
        let results = crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Delete,
                component: db::Component::Inventory,
            },
            pool,
            "DELETE FROM inventory_items
            WHERE
                id = $1
                AND user_id = $2",
            id,
            ctx.user.id
        )
        .await?;

        Ok(results.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn update(
        ctx: &Context,
        pool: &db::Pool,
        id: Uuid,
        name: &str,
        weight: u32,
    ) -> Result<Uuid, Error> {
        let weight = i32::try_from(weight).unwrap();
        crate::execute_returning_uuid!(
            &db::QueryClassification {
                query_type: db::QueryType::Update,
                component: db::Component::Inventory,
            },
            pool,
            "UPDATE inventory_items AS item
            SET
                name = $1,
                weight = $2
            WHERE
                item.id = $3
                AND item.user_id = $4
            RETURNING item.category_id AS id
            ",
            name,
            weight,
            id,
            ctx.user.id
        )
        .await
    }

    #[tracing::instrument]
    pub async fn save(
        ctx: &Context,
        pool: &db::Pool,
        name: &str,
        category_id: Uuid,
        weight: u32,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let weight = i32::try_from(weight).unwrap();

        crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Insert,
                component: db::Component::Inventory,
            },
            pool,
            "INSERT INTO inventory_items
                (id, name, description, weight, category_id, user_id)
            VALUES
                ($1, $2, $3, $4, $5, $6)",
            id,
            name,
            "",
            weight,
            category_id,
            ctx.user.id
        )
        .await?;

        Ok(id)
    }

    #[tracing::instrument]
    pub async fn get_category_max_weight(
        ctx: &Context,
        pool: &db::Pool,
        category_id: Uuid,
    ) -> Result<i32, Error> {
        let weight = crate::execute_returning!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Inventory,
            },
            pool,
            "
                SELECT COALESCE(MAX(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                WHERE
                    category_id = $1
                    AND category.user_id = $2
            ",
            i32,
            |row| row.weight.unwrap(),
            category_id,
            ctx.user.id
        )
        .await?;

        Ok(weight)
    }
}

#[derive(Debug)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub weight: i32,
    pub category_id: Uuid,
}

pub struct DbInventoryItemsRow {
    pub id: Uuid,
    pub name: String,
    pub weight: i32,
    pub description: Option<String>,
    pub category_id: Uuid,
}

impl TryFrom<DbInventoryItemsRow> for Item {
    type Error = Error;

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
    #[tracing::instrument]
    pub async fn _get_category_total_picked_weight(
        ctx: &Context,
        pool: &db::Pool,
        category_id: Uuid,
    ) -> Result<i32, Error> {
        crate::execute_returning!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Inventory,
            },
            pool,
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
