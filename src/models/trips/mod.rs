use std::fmt;

use crate::components::crud::*;

use super::{
    error::{DatabaseError, Error, QueryError},
    inventory,
};

use crate::{db, Context};

use serde::{Deserialize, Serialize};
use time;
use uuid::Uuid;

#[derive(PartialEq, PartialOrd, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "trip_state")]
#[sqlx(rename_all = "lowercase")]
pub enum TripState {
    Init,
    Planning,
    Planned,
    Active,
    Review,
    Done,
}

#[allow(clippy::new_without_default)]
impl TripState {
    pub fn new() -> Self {
        TripState::Init
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Init => Some(Self::Planning),
            Self::Planning => Some(Self::Planned),
            Self::Planned => Some(Self::Active),
            Self::Active => Some(Self::Review),
            Self::Review => Some(Self::Done),
            Self::Done => None,
        }
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            Self::Init => None,
            Self::Planning => Some(Self::Init),
            Self::Planned => Some(Self::Planning),
            Self::Active => Some(Self::Planned),
            Self::Review => Some(Self::Active),
            Self::Done => Some(Self::Review),
        }
    }
}

impl fmt::Display for TripState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Init => "Init",
                Self::Planning => "Planning",
                Self::Planned => "Planned",
                Self::Active => "Active",
                Self::Review => "Review",
                Self::Done => "Done",
            },
        )
    }
}

impl std::convert::TryFrom<&str> for TripState {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Init" => Self::Init,
            "Planning" => Self::Planning,
            "Planned" => Self::Planned,
            "Active" => Self::Active,
            "Review" => Self::Review,
            "Done" => Self::Done,
            _ => {
                return Err(Error::Database(DatabaseError::Enum {
                    description: format!("{value} is not a valid value for TripState"),
                }))
            }
        })
    }
}

#[derive(Serialize, Debug)]
pub enum TripItemStateKey {
    Pick,
    Pack,
    Ready,
}

impl fmt::Display for TripItemStateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pick => "pick",
                Self::Pack => "pack",
                Self::Ready => "ready",
            },
        )
    }
}

#[derive(Debug)]
pub struct TripCategory {
    pub category: inventory::Category,
    pub items: Option<Vec<TripItem>>,
}

impl TripCategory {
    #[tracing::instrument]
    pub fn total_picked_weight(&self) -> i32 {
        self.items
            .as_ref()
            .unwrap()
            .iter()
            .filter(|item| item.picked)
            .map(|item| item.item.weight)
            .sum()
    }

    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &db::Pool,
        trip_id: Uuid,
        category_id: Uuid,
    ) -> Result<Option<TripCategory>, Error> {
        struct Row {
            category_id: Uuid,
            category_name: String,
            category_description: Option<String>,
            #[allow(dead_code)]
            trip_id: Option<Uuid>,
            item_id: Option<Uuid>,
            item_name: Option<String>,
            item_description: Option<String>,
            item_weight: Option<i32>,
            item_is_picked: Option<bool>,
            item_is_packed: Option<bool>,
            item_is_ready: Option<bool>,
            item_is_new: Option<bool>,
        }

        struct RowParsed {
            category: TripCategory,
            item: Option<TripItem>,
        }

        impl TryFrom<Row> for RowParsed {
            type Error = Error;

            fn try_from(row: Row) -> Result<Self, Self::Error> {
                let category = inventory::Category {
                    id: row.category_id,
                    name: row.category_name,
                    description: row.category_description,
                    items: None,
                };
                Ok(Self {
                    category: TripCategory {
                        category,
                        items: None,
                    },

                    item: match row.item_id {
                        Some(item_id) => Some(TripItem {
                            item: inventory::Item {
                                id: item_id,
                                name: row.item_name.unwrap(),
                                description: row.item_description,
                                weight: row.item_weight.unwrap().try_into().unwrap(),
                                category_id: row.category_id,
                            },
                            picked: row.item_is_picked.unwrap(),
                            packed: row.item_is_packed.unwrap(),
                            ready: row.item_is_ready.unwrap(),
                            new: row.item_is_new.unwrap(),
                        }),
                        None => None,
                    },
                })
            }
        }

        let mut rows = crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            Row,
            RowParsed,
            r#"
                WITH category_items AS (
                     SELECT
                        trip.trip_id AS trip_id,
                        category.id AS category_id,
                        category.name AS category_name,
                        category.description AS category_description,
                        item.id AS item_id,
                        item.name AS item_name,
                        item.description AS item_description,
                        item.weight AS item_weight,
                        trip.pick AS item_is_picked,
                        trip.pack AS item_is_packed,
                        trip.ready AS item_is_ready,
                        trip.new AS item_is_new
                    FROM trip_items AS trip
                    INNER JOIN inventory_items AS item
                        ON item.id = trip.item_id
                    INNER JOIN inventory_items_categories AS category
                        ON category.id = item.category_id
                    WHERE
                        trip.trip_id = $1
                        AND trip.user_id = $2                   
                )
                SELECT
                    category.id AS category_id,
                    category.name AS category_name,
                    category.description AS category_description,
                    items.trip_id AS trip_id,
                    items.item_id AS item_id,
                    items.item_name AS item_name,
                    items.item_description AS item_description,
                    items.item_weight AS item_weight,
                    items.item_is_picked AS item_is_picked,
                    items.item_is_packed AS item_is_packed,
                    items.item_is_ready AS item_is_ready,
                    items.item_is_new AS item_is_new
                FROM inventory_items_categories AS category
                    LEFT JOIN category_items AS items
                    ON items.category_id = category.id
                WHERE category.id = $3
            "#,
            trip_id,
            ctx.user.id,
            category_id
        )
        .await?;

        let mut category = match rows.pop() {
            None => return Ok(None),
            Some(initial) => TripCategory {
                category: initial.category.category,
                items: initial.item.map(|item| vec![item]).or_else(|| Some(vec![])),
            },
        };

        for row in rows {
            let item = row.item;
            category.items = category.items.or_else(|| Some(vec![]));

            if let Some(item) = item {
                category.items = category.items.map(|mut c| {
                    c.push(item);
                    c
                });
            };
        }

        Ok(Some(category))
    }
}

// TODO refactor the bools into an enum
#[derive(Debug)]
pub struct TripItem {
    pub item: inventory::Item,
    pub picked: bool,
    pub packed: bool,
    pub ready: bool,
    pub new: bool,
}

pub struct DbTripsItemsRow {
    pub picked: bool,
    pub packed: bool,
    pub ready: bool,
    pub new: bool,
    pub id: Uuid,
    pub name: String,
    pub weight: i32,
    pub description: Option<String>,
    pub category_id: Uuid,
}

impl TryFrom<DbTripsItemsRow> for TripItem {
    type Error = Error;

    fn try_from(row: DbTripsItemsRow) -> Result<Self, Self::Error> {
        Ok(TripItem {
            picked: row.picked,
            packed: row.packed,
            ready: row.ready,
            new: row.new,
            item: inventory::Item {
                id: row.id,
                name: row.name,
                description: row.description,
                weight: row.weight,
                category_id: row.category_id,
            },
        })
    }
}

impl TripItem {
    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &db::Pool,
        trip_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        crate::query_one!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            DbTripsItemsRow,
            Self,
            "
                SELECT
                    t_item.item_id AS id,
                    t_item.pick AS picked,
                    t_item.pack AS packed,
                    t_item.ready AS ready,
                    t_item.new AS new,
                    i_item.name AS name,
                    i_item.description AS description,
                    i_item.weight AS weight,
                    i_item.category_id AS category_id
                FROM trip_items AS t_item
                INNER JOIN inventory_items AS i_item
                    ON i_item.id = t_item.item_id
                WHERE t_item.item_id = $1
                AND t_item.trip_id = $2
                AND t_item.user_id = $3
            ",
            item_id,
            trip_id,
            ctx.user.id
        )
        .await
    }

    #[tracing::instrument]
    pub async fn set_state(
        ctx: &Context,
        pool: &db::Pool,
        trip_id: Uuid,
        item_id: Uuid,
        key: TripItemStateKey,
        value: bool,
    ) -> Result<(), Error> {
        let result = match key {
            TripItemStateKey::Pick => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trip_items
                        SET pick = $1
                        WHERE trip_id = $2
                        AND item_id = $3
                        AND user_id = $4",
                    value,
                    trip_id,
                    item_id,
                    ctx.user.id
                )
                .await
            }
            TripItemStateKey::Pack => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trip_items
                        SET pack = $1
                        WHERE trip_id = $2
                        AND item_id = $3
                        AND user_id = $4",
                    value,
                    trip_id,
                    item_id,
                    ctx.user.id
                )
                .await
            }
            TripItemStateKey::Ready => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trip_items
                        SET ready = $1
                        WHERE trip_id = $2
                        AND item_id = $3
                        AND user_id = $4",
                    value,
                    trip_id,
                    item_id,
                    ctx.user.id
                )
                .await
            }
        }?;

        (result.rows_affected() != 0).then_some(()).ok_or_else(|| {
            Error::Query(QueryError::NotFound {
                description: format!("item {item_id} not found for trip {trip_id}"),
            })
        })
    }
}

pub struct DbTripRow {
    pub id: Uuid,
    pub name: String,
    pub date_start: time::Date,
    pub date_end: time::Date,
    pub state: TripState,
    pub location: Option<String>,
    pub temp_min: Option<i32>,
    pub temp_max: Option<i32>,
    pub comment: Option<String>,
}

impl TryFrom<DbTripRow> for Trip {
    type Error = Error;

    fn try_from(row: DbTripRow) -> Result<Self, Self::Error> {
        Ok(Trip {
            id: row.id,
            name: row.name,
            date_start: row.date_start,
            date_end: row.date_end,
            state: row.state,
            location: row.location,
            temp_min: row.temp_min,
            temp_max: row.temp_max,
            comment: row.comment,
            todos: None,
            types: None,
            categories: None,
        })
    }
}

#[derive(Debug)]
pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub date_start: time::Date,
    pub date_end: time::Date,
    pub state: TripState,
    pub location: Option<String>,
    pub temp_min: Option<i32>,
    pub temp_max: Option<i32>,
    pub comment: Option<String>,
    pub todos: Option<Vec<crate::components::trips::todos::Todo>>,
    pub types: Option<Vec<TripType>>,
    pub categories: Option<Vec<TripCategory>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TripAttributeUpdate {
    #[serde(rename = "name")]
    Name(String),
    #[serde(rename = "date_start")]
    DateStart(time::Date),
    #[serde(rename = "date_end")]
    DateEnd(time::Date),
    #[serde(rename = "location")]
    Location(String),
    #[serde(rename = "temp_min")]
    TempMin(i32),
    #[serde(rename = "temp_max")]
    TempMax(i32),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TripAttribute {
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "date_start")]
    DateStart,
    #[serde(rename = "date_end")]
    DateEnd,
    #[serde(rename = "location")]
    Location,
    #[serde(rename = "temp_min")]
    TempMin,
    #[serde(rename = "temp_max")]
    TempMax,
}

impl Trip {
    #[tracing::instrument]
    pub async fn all(ctx: &Context, pool: &db::Pool) -> Result<Vec<Trip>, Error> {
        let mut trips = crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            DbTripRow,
            Self,
            r#"SELECT
                id,
                name,
                date_start,
                date_end,
                state as "state: _",
                location,
                temp_min,
                temp_max,
                comment
            FROM trips
            WHERE user_id = $1"#,
            ctx.user.id
        )
        .await?;

        trips.sort_by_key(|trip| trip.date_start);
        Ok(trips)
    }

    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &db::Pool,
        trip_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        crate::query_one!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            DbTripRow,
            Self,
            r#"SELECT
                id,
                name,
                date_start,
                date_end,
                state as "state: _",
                location,
                temp_min,
                temp_max,
                comment
            FROM trips
            WHERE id = $1 and user_id = $2"#,
            trip_id,
            ctx.user.id
        )
        .await
    }

    #[tracing::instrument]
    pub async fn trip_type_remove(
        ctx: &Context,
        pool: &db::Pool,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<bool, Error> {
        let results = crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Delete,
                component: db::Component::Trips,
            },
            pool,
            "DELETE FROM trip_to_trip_types AS ttt
            WHERE ttt.trip_id = $1
                AND ttt.trip_type_id = $2
            AND EXISTS(SELECT * FROM trips WHERE id = $1 AND user_id = $3)
            AND EXISTS(SELECT * FROM trip_types WHERE id = $2 AND user_id = $3)
            ",
            id,
            type_id,
            ctx.user.id
        )
        .await?;

        Ok(results.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn trip_type_add(
        ctx: &Context,
        pool: &db::Pool,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<(), Error> {
        // TODO user handling?

        crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Insert,
                component: db::Component::Trips,
            },
            pool,
            "INSERT INTO
                trip_to_trip_types (trip_id, trip_type_id)
            (SELECT trips.id as trip_id, trip_types.id as trip_type_id
                FROM trips
                INNER JOIN trip_types ON true
                WHERE
                    trips.id = $1
                    AND trips.user_id = $3
                    AND trip_types.id = $2
                    AND trip_types.user_id = $3)",
            id,
            type_id,
            ctx.user.id
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn set_state(
        ctx: &Context,
        pool: &db::Pool,
        id: Uuid,
        new_state: &TripState,
    ) -> Result<bool, Error> {
        let result = crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Update,
                component: db::Component::Trips,
            },
            pool,
            "UPDATE trips
            SET state = $1
            WHERE id = $2 and user_id = $3",
            new_state as _,
            id,
            ctx.user.id
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn set_comment(
        ctx: &Context,
        pool: &db::Pool,
        id: Uuid,
        new_comment: &str,
    ) -> Result<bool, Error> {
        let result = crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Update,
                component: db::Component::Trips,
            },
            pool,
            "UPDATE trips
            SET comment = $1
            WHERE id = $2 AND user_id = $3",
            new_comment,
            id,
            ctx.user.id
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn set_attribute(
        ctx: &Context,
        pool: &db::Pool,
        trip_id: Uuid,
        attribute: TripAttributeUpdate,
    ) -> Result<(), Error> {
        let result = match attribute {
            TripAttributeUpdate::Name(value) => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET name = $1
                WHERE id = $2 AND user_id = $3",
                    value,
                    trip_id,
                    ctx.user.id
                )
                .await
            }

            TripAttributeUpdate::DateStart(value) => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET date_start = $1
                WHERE id = $2 AND user_id = $3",
                    value,
                    trip_id,
                    ctx.user.id
                )
                .await
            }
            TripAttributeUpdate::DateEnd(value) => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET date_end = $1
                WHERE id = $2 AND user_id = $3",
                    value,
                    trip_id,
                    ctx.user.id
                )
                .await
            }
            TripAttributeUpdate::Location(value) => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET location = $1
                WHERE id = $2 AND user_id = $3",
                    value,
                    trip_id,
                    ctx.user.id
                )
                .await
            }
            TripAttributeUpdate::TempMin(value) => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET temp_min = $1
                WHERE id = $2 AND user_id = $3",
                    value,
                    trip_id,
                    ctx.user.id
                )
                .await
            }
            TripAttributeUpdate::TempMax(value) => {
                crate::execute!(
                    &db::QueryClassification {
                        query_type: db::QueryType::Update,
                        component: db::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET temp_max = $1
                WHERE id = $2 AND user_id = $3",
                    value,
                    trip_id,
                    ctx.user.id
                )
                .await
            }
        }?;

        (result.rows_affected() != 0).then_some(()).ok_or_else(|| {
            Error::Query(QueryError::NotFound {
                description: format!("trip {trip_id} not found"),
            })
        })
    }

    #[tracing::instrument]
    pub async fn save(
        ctx: &Context,
        pool: &db::Pool,
        name: &str,
        date_start: time::Date,
        date_end: time::Date,
        copy_from: Option<Uuid>,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();

        let trip_state = TripState::new();

        let mut transaction = pool.begin().await?;

        crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Insert,
                component: db::Component::Trips,
            },
            &mut *transaction,
            "INSERT INTO trips
                (id, name, date_start, date_end, state, user_id)
            VALUES
                ($1, $2, $3, $4, $5, $6)",
            id,
            name,
            date_start,
            date_end,
            trip_state as _,
            ctx.user.id,
        )
        .await?;

        if let Some(copy_from_trip_id) = copy_from {
            crate::execute!(
                &db::QueryClassification {
                    query_type: db::QueryType::Insert,
                    component: db::Component::Trips,
                },
                &mut *transaction,
                r#"INSERT INTO trip_items (
                    item_id,
                    trip_id,
                    pick,
                    pack,
                    ready,
                    new,
                    user_id
                ) SELECT
                    item_id,
                    $1 as trip_id,
                    pick,
                    false as pack,
                    false as ready,
                    false as new,
                    user_id
                FROM trip_items
                WHERE trip_id = $2 AND user_id = $3"#,
                id,
                copy_from_trip_id,
                ctx.user.id
            )
            .await?;
        } else {
            crate::execute!(
                &db::QueryClassification {
                    query_type: db::QueryType::Insert,
                    component: db::Component::Trips,
                },
                &mut *transaction,
                r#"INSERT INTO trip_items (
                    item_id,
                    trip_id,
                    pick,
                    pack,
                    ready,
                    new,
                    user_id
                ) SELECT
                    id as item_id,
                    $1 as trip_id,
                    false as pick,
                    false as pack,
                    false as ready,
                    false as new,
                    user_id
                FROM inventory_items
                WHERE user_id = $2"#,
                id,
                ctx.user.id
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(id)
    }

    #[tracing::instrument]
    pub async fn find_total_picked_weight(
        ctx: &Context,
        pool: &db::Pool,
        trip_id: Uuid,
    ) -> Result<i32, Error> {
        let weight = crate::execute_returning!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            "
                SELECT
                    CAST(COALESCE(SUM(i_item.weight), 0) AS INTEGER) AS total_weight
                FROM trips AS trip
                INNER JOIN trip_items AS t_item
                    ON t_item.trip_id = trip.id
                INNER JOIN inventory_items AS i_item
                    ON t_item.item_id = i_item.id
                WHERE
                    trip.id = $1 AND trip.user_id = $2
                AND t_item.pick = true
            ",
            i32,
            |row| row.total_weight.unwrap(),
            trip_id,
            ctx.user.id
        )
        .await?;

        Ok(weight)
    }

    #[tracing::instrument]
    pub fn types(&self) -> &Vec<TripType> {
        self.types
            .as_ref()
            .expect("you need to call load_trip_types()")
    }

    #[tracing::instrument]
    pub fn categories(&self) -> &Vec<TripCategory> {
        self.categories
            .as_ref()
            .expect("you need to call load_categories()")
    }

    #[tracing::instrument]
    pub fn todos(&self) -> &Vec<crate::components::trips::todos::Todo> {
        self.todos.as_ref().expect("you need to call load_todos()")
    }

    #[tracing::instrument]
    pub fn total_picked_weight(&self) -> i32 {
        self.categories()
            .iter()
            .map(|category| -> i32 {
                category
                    .items
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter_map(|item| Some(item.item.weight).filter(|_| item.picked))
                    .sum::<i32>()
            })
            .sum::<i32>()
    }

    #[tracing::instrument]
    pub async fn load_todos(&mut self, ctx: &Context, pool: &db::Pool) -> Result<(), Error> {
        self.todos = Some(
            crate::components::trips::todos::Todo::findall(
                ctx,
                pool,
                crate::components::trips::todos::Container { trip_id: self.id },
            )
            .await?,
        );
        Ok(())
    }

    #[tracing::instrument]
    pub async fn load_trip_types(&mut self, ctx: &Context, pool: &db::Pool) -> Result<(), Error> {
        let types = crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            TripTypeRow,
            TripType,
            r#"
            WITH trips AS (
                SELECT type.id as id, trip.user_id as user_id
                FROM trips as trip
                INNER JOIN trip_to_trip_types as ttt
                    ON ttt.trip_id = trip.id
                INNER JOIN trip_types AS type
                    ON type.id = ttt.trip_type_id
                WHERE trip.id = $1 AND trip.user_id = $2
            )
            SELECT
                type.id AS id,
                type.name AS name,
                trips.id IS NOT NULL AS "active!"
            FROM trip_types AS type
                LEFT JOIN trips
                ON trips.id = type.id
            WHERE type.user_id = $2
            "#,
            self.id,
            ctx.user.id
        )
        .await?;

        self.types = Some(types);
        Ok(())
    }

    #[tracing::instrument]
    pub async fn sync_trip_items_with_inventory(
        &mut self,
        ctx: &Context,
        pool: &db::Pool,
    ) -> Result<(), Error> {
        // we need to get all items that are part of the inventory but not
        // part of the trip items
        //
        // then, we know which items we need to sync. there are different
        // states for them:
        //
        // * if the trip is new (it's state is INITIAL), we can just forward
        //   as-is
        // * if the trip is not new, we have to make these new items prominently
        //   visible so the user knows that there might be new items to
        //   consider
        struct Row {
            item_id: Uuid,
        }

        impl TryFrom<Row> for Uuid {
            type Error = Error;

            fn try_from(value: Row) -> Result<Self, Self::Error> {
                Ok(value.item_id)
            }
        }

        let unsynced_items: Vec<Uuid> = crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            Row,
            Uuid,
            "
            SELECT
                i_item.id AS item_id
            FROM inventory_items AS i_item
                LEFT JOIN (
                    SELECT t_item.item_id AS item_id, t_item.user_id AS user_id
                    FROM trip_items AS t_item
                    WHERE t_item.trip_id = $1 AND t_item.user_id = $2
                ) AS t_item
                ON t_item.item_id = i_item.id
            WHERE t_item.item_id IS NULL AND i_item.user_id = $2",
            self.id,
            ctx.user.id
        )
        .await?;

        // only mark as new when the trip not already underway
        let mark_as_new = self.state < TripState::Active;

        for unsynced_item in &unsynced_items {
            crate::execute!(
                &db::QueryClassification {
                    query_type: db::QueryType::Insert,
                    component: db::Component::Trips,
                },
                pool,
                "
                INSERT INTO trip_items
                    (
                        item_id,
                        trip_id,
                        pick,
                        pack,
                        ready,
                        new,
                        user_id
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                ",
                unsynced_item,
                self.id,
                false,
                false,
                false,
                mark_as_new,
                ctx.user.id
            )
            .await?;
        }

        tracing::info!("unsynced items: {:?}", &unsynced_items);

        Ok(())
    }

    #[tracing::instrument]
    pub async fn load_categories(&mut self, ctx: &Context, pool: &db::Pool) -> Result<(), Error> {
        let mut categories: Vec<TripCategory> = vec![];
        // we can ignore the return type as we collect into `categories`
        // in the `map_ok()` closure
        struct Row {
            category_id: Uuid,
            category_name: String,
            category_description: Option<String>,
            #[allow(dead_code)]
            trip_id: Option<Uuid>,
            item_id: Option<Uuid>,
            item_name: Option<String>,
            item_description: Option<String>,
            item_weight: Option<i32>,
            item_is_picked: Option<bool>,
            item_is_packed: Option<bool>,
            item_is_ready: Option<bool>,
            item_is_new: Option<bool>,
        }

        struct RowParsed {
            category: TripCategory,
            item: Option<TripItem>,
        }

        impl TryFrom<Row> for RowParsed {
            type Error = Error;

            fn try_from(row: Row) -> Result<Self, Self::Error> {
                let category = inventory::Category {
                    id: row.category_id,
                    name: row.category_name,
                    description: row.category_description,
                    items: None,
                };
                Ok(Self {
                    category: TripCategory {
                        category,
                        items: None,
                    },

                    item: match row.item_id {
                        Some(item_id) => Some(TripItem {
                            item: inventory::Item {
                                id: item_id,
                                name: row.item_name.unwrap(),
                                description: row.item_description,
                                weight: row.item_weight.unwrap(),
                                category_id: row.category_id,
                            },
                            picked: row.item_is_picked.unwrap(),
                            packed: row.item_is_packed.unwrap(),
                            ready: row.item_is_ready.unwrap(),
                            new: row.item_is_new.unwrap(),
                        }),
                        None => None,
                    },
                })
            }
        }

        let rows = crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            Row,
            RowParsed,
            r#"
                WITH trip_items AS (
                    SELECT
                        trip.trip_id AS trip_id,
                        category.id AS category_id,
                        category.name AS category_name,
                        category.description AS category_description,
                        item.id AS item_id,
                        item.name AS item_name,
                        item.description AS item_description,
                        item.weight AS item_weight,
                        trip.pick AS item_is_picked,
                        trip.pack AS item_is_packed,
                        trip.ready AS item_is_ready,
                        trip.new AS item_is_new,
                        trip.user_id AS user_id
                    FROM trip_items AS trip
                    INNER JOIN inventory_items AS item
                        ON item.id = trip.item_id
                    INNER JOIN inventory_items_categories AS category
                        ON category.id = item.category_id
                    WHERE trip.trip_id = $1 AND trip.user_id = $2
                    
                )
                SELECT
                    category.id AS category_id,
                    category.name AS category_name,
                    category.description AS category_description,
                    trip_items.trip_id AS trip_id,
                    trip_items.item_id AS item_id,
                    trip_items.item_name AS item_name,
                    trip_items.item_description AS item_description,
                    trip_items.item_weight AS item_weight,
                    trip_items.item_is_picked AS item_is_picked,
                    trip_items.item_is_packed AS item_is_packed,
                    trip_items.item_is_ready AS item_is_ready,
                    trip_items.item_is_new AS item_is_new
                FROM inventory_items_categories AS category
                    LEFT JOIN trip_items
                    ON trip_items.category_id = category.id
                WHERE category.user_id = $2
            "#,
            self.id,
            ctx.user.id
        )
        .await?;

        for row in rows {
            match categories
                .iter_mut()
                .find(|cat| cat.category.id == row.category.category.id)
            {
                Some(ref mut existing_category) => {
                    // taking and then readding later
                    let mut items = existing_category.items.take().unwrap_or(vec![]);

                    if let Some(item) = row.item {
                        items.push(item);
                    }

                    existing_category.items = Some(items);
                }
                None => categories.push(TripCategory {
                    category: row.category.category,
                    items: row.item.map(|item| vec![item]).or_else(|| Some(vec![])),
                }),
            }
        }

        self.categories = Some(categories);

        Ok(())
        // .fetch(pool)
        // .map_ok(|row| -> Result<(), Error> {
        //     let mut category = TripCategory {
        //         category: inventory::Category {
        //             id: Uuid::try_parse(&row.category_id)?,
        //             name: row.category_name,
        //             description: row.category_description,

        //             items: None,
        //         },
        //         items: None,
        //     };

        //     match row.item_id {
        //         None => {
        //             // we have an empty (unused) category which has NULL values
        //             // for the item_id column
        //             category.items = Some(vec![]);
        //             categories.push(category);
        //         }
        //         Some(item_id) => {
        //             let item = TripItem {
        //                 item: inventory::Item {
        //                     id: Uuid::try_parse(&item_id)?,
        //                     name: row.item_name.unwrap(),
        //                     description: row.item_description,
        //                     weight: row.item_weight.unwrap(),
        //                     category_id: category.category.id,
        //                 },
        //                 picked: row.item_is_picked.unwrap(),
        //                 packed: row.item_is_packed.unwrap(),
        //                 ready: row.item_is_ready.unwrap(),
        //                 new: row.item_is_new.unwrap(),
        //             };

        //             if let Some(&mut ref mut c) = categories
        //                 .iter_mut()
        //                 .find(|c| c.category.id == category.category.id)
        //             {
        //                 // we always populate c.items when we add a new category, so
        //                 // it's safe to unwrap here
        //                 c.items.as_mut().unwrap().push(item);
        //             } else {
        //                 category.items = Some(vec![item]);
        //                 categories.push(category);
        //             }
        //         }
        //     }

        //     Ok(())
        // })
        // .try_collect::<Vec<Result<(), Error>>>()
        // .await?
        // .into_iter()
        // .collect::<Result<(), Error>>()?;
    }
}

#[derive(Debug)]
pub struct TripType {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
}

struct TripTypeRow {
    id: Uuid,
    name: String,
    active: bool,
}

impl TryFrom<TripTypeRow> for TripType {
    type Error = Error;

    fn try_from(row: TripTypeRow) -> Result<Self, Self::Error> {
        Ok(TripType {
            id: row.id,
            name: row.name,
            active: row.active,
        })
    }
}

impl TripsType {
    #[tracing::instrument]
    pub async fn all(ctx: &Context, pool: &db::Pool) -> Result<Vec<Self>, Error> {
        crate::query_all!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Trips,
            },
            pool,
            DbTripsTypesRow,
            Self,
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
    pub async fn save(ctx: &Context, pool: &db::Pool, name: &str) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Insert,
                component: db::Component::Trips,
            },
            pool,
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
        pool: &db::Pool,
        id: Uuid,
        new_name: &str,
    ) -> Result<bool, Error> {
        let result = crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Update,
                component: db::Component::Trips,
            },
            pool,
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

pub(crate) struct DbTripsTypesRow {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TripTypeAttribute {
    #[serde(rename = "name")]
    Name,
}

#[derive(Debug)]
pub struct TripsType {
    pub id: Uuid,
    pub name: String,
}

impl TryFrom<DbTripsTypesRow> for TripsType {
    type Error = Error;

    fn try_from(row: DbTripsTypesRow) -> Result<Self, Self::Error> {
        Ok(TripsType {
            id: row.id,
            name: row.name,
        })
    }
}
