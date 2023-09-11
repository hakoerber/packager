use std::fmt;

use super::{
    consts,
    error::{DatabaseError, Error, QueryError},
    inventory,
};

use crate::{sqlite, Context};

use futures::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use time;
use uuid::Uuid;

// #[macro_use]
// mod macros {
//     macro_rules! build_state_query {
//         ($pool:expr, $state_name:literal, $value:expr, $trip_id:expr, $item_id:expr, $user_id:expr) => {
//             crate::execute!(
//                 &sqlite::QueryClassification {
//                     query_type: sqlite::QueryType::Update,
//                     component: sqlite::Component::Trips,
//                 },
//                 $pool,
//                 ["UPDATE trips_items SET pick = ?
//                 WHERE trip_id = ?
//                 AND item_id = ?
//                 AND user_id = "
//                     ,"?"]
//                 $value,
//                 $trip_id,
//                 $item_id,
//                 $user_id
//             )
//         };
//     }
// }

#[derive(sqlite::Type, PartialEq, PartialOrd, Deserialize, Debug)]
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
    pub fn total_picked_weight(&self) -> i64 {
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
        pool: &sqlite::Pool,
        trip_id: Uuid,
        category_id: Uuid,
    ) -> Result<Option<TripCategory>, Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = trip_id.to_string();
        let category_id_param = category_id.to_string();

        struct Row {
            category_id: String,
            category_name: String,
            category_description: Option<String>,
            #[allow(dead_code)]
            trip_id: Option<String>,
            item_id: Option<String>,
            item_name: Option<String>,
            item_description: Option<String>,
            item_weight: Option<i64>,
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
                    id: Uuid::try_parse(&row.category_id)?,
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
                                id: Uuid::try_parse(&item_id)?,
                                name: row.item_name.unwrap(),
                                description: row.item_description,
                                weight: row.item_weight.unwrap(),
                                category_id: Uuid::try_parse(&row.category_id)?,
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
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
            },
            pool,
            Row,
            RowParsed,
            r#"
                SELECT
                    category.id as category_id,
                    category.name as category_name,
                    category.description AS category_description,
                    inner.trip_id AS trip_id,
                    inner.item_id AS item_id,
                    inner.item_name AS item_name,
                    inner.item_description AS item_description,
                    inner.item_weight AS item_weight,
                    inner.item_is_picked AS item_is_picked,
                    inner.item_is_packed AS item_is_packed,
                    inner.item_is_ready AS item_is_ready,
                    inner.item_is_new AS item_is_new
                FROM inventory_items_categories AS category
                    LEFT JOIN (
                        SELECT
                            trip.trip_id AS trip_id,
                            category.id as category_id,
                            category.name as category_name,
                            category.description as category_description,
                            item.id as item_id,
                            item.name as item_name,
                            item.description as item_description,
                            item.weight as item_weight,
                            trip.pick as item_is_picked,
                            trip.pack as item_is_packed,
                            trip.ready as item_is_ready,
                            trip.new as item_is_new
                        FROM trips_items as trip
                        INNER JOIN inventory_items as item
                            ON item.id = trip.item_id
                        INNER JOIN inventory_items_categories as category
                            ON category.id = item.category_id
                        WHERE 
                            trip.trip_id = ?
                            AND trip.user_id = ?
                    ) AS inner
                    ON inner.category_id = category.id
                WHERE category.id = ?
            "#,
            trip_id_param,
            user_id,
            category_id_param
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
    pub id: String,
    pub name: String,
    pub weight: i64,
    pub description: Option<String>,
    pub category_id: String,
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
                id: Uuid::try_parse(&row.id)?,
                name: row.name,
                description: row.description,
                weight: row.weight,
                category_id: Uuid::try_parse(&row.category_id)?,
            },
        })
    }
}

impl TripItem {
    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let user_id = ctx.user.id.to_string();
        let item_id_param = item_id.to_string();
        let trip_id_param = trip_id.to_string();
        crate::query_one!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
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
                FROM trips_items AS t_item
                INNER JOIN inventory_items AS i_item
                    ON i_item.id = t_item.item_id
                WHERE t_item.item_id = ?
                AND t_item.trip_id = ?
                AND t_item.user_id = ?
            ",
            item_id_param,
            trip_id_param,
            user_id,
        )
        .await
    }

    #[tracing::instrument]
    pub async fn set_state(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
        item_id: Uuid,
        key: TripItemStateKey,
        value: bool,
    ) -> Result<(), Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = trip_id.to_string();
        let item_id_param = item_id.to_string();
        let result = match key {
            TripItemStateKey::Pick => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips_items
                        SET pick = ?
                        WHERE trip_id = ?
                        AND item_id = ?
                        AND user_id = ?",
                    value,
                    trip_id_param,
                    item_id_param,
                    user_id
                )
                .await
            }
            TripItemStateKey::Pack => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips_items
                        SET pack = ?
                        WHERE trip_id = ?
                        AND item_id = ?
                        AND user_id = ?",
                    value,
                    trip_id_param,
                    item_id_param,
                    user_id
                )
                .await
            }
            TripItemStateKey::Ready => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips_items
                        SET ready = ?
                        WHERE trip_id = ?
                        AND item_id = ?
                        AND user_id = ?",
                    value,
                    trip_id_param,
                    item_id_param,
                    user_id
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
    pub id: String,
    pub name: String,
    pub date_start: String,
    pub date_end: String,
    pub state: String,
    pub location: Option<String>,
    pub temp_min: Option<i64>,
    pub temp_max: Option<i64>,
    pub comment: Option<String>,
}

impl TryFrom<DbTripRow> for Trip {
    type Error = Error;

    fn try_from(row: DbTripRow) -> Result<Self, Self::Error> {
        Ok(Trip {
            id: Uuid::try_parse(&row.id)?,
            name: row.name,
            date_start: time::Date::parse(&row.date_start, consts::DATE_FORMAT)?,
            date_end: time::Date::parse(&row.date_end, consts::DATE_FORMAT)?,
            state: row.state.as_str().try_into()?,
            location: row.location,
            temp_min: row.temp_min,
            temp_max: row.temp_max,
            comment: row.comment,
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
    pub temp_min: Option<i64>,
    pub temp_max: Option<i64>,
    pub comment: Option<String>,
    pub(crate) types: Option<Vec<TripType>>,
    pub(crate) categories: Option<Vec<TripCategory>>,
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
    pub async fn all(ctx: &Context, pool: &sqlite::Pool) -> Result<Vec<Trip>, Error> {
        let user_id = ctx.user.id.to_string();
        let mut trips = crate::query_all!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
            },
            pool,
            DbTripRow,
            Self,
            "SELECT
                id,
                name,
                CAST (date_start AS TEXT) date_start,
                CAST (date_end AS TEXT) date_end,
                state,
                location,
                temp_min,
                temp_max,
                comment
            FROM trips
            WHERE user_id = ?",
            user_id
        )
        .await?;

        trips.sort_by_key(|trip| trip.date_start);
        Ok(trips)
    }

    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let trip_id_param = trip_id.to_string();
        let user_id = ctx.user.id.to_string();
        crate::query_one!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
            },
            pool,
            DbTripRow,
            Self,
            "SELECT
                id,
                name,
                CAST (date_start AS TEXT) date_start,
                CAST (date_end AS TEXT) date_end,
                state,
                location,
                temp_min,
                temp_max,
                comment
            FROM trips
            WHERE id = ? and user_id = ?",
            trip_id_param,
            user_id,
        )
        .await
    }

    #[tracing::instrument]
    pub async fn trip_type_remove(
        ctx: &Context,
        pool: &sqlite::Pool,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<bool, Error> {
        let user_id = ctx.user.id.to_string();
        let id_param = id.to_string();
        let type_id_param = type_id.to_string();

        let results = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Delete,
                component: sqlite::Component::Trips,
            },
            pool,
            "DELETE FROM trips_to_trips_types AS ttt
            WHERE ttt.trip_id = ?
                AND ttt.trip_type_id = ?
            AND EXISTS(SELECT * FROM trips WHERE id = ? AND user_id = ?)
            AND EXISTS(SELECT * FROM trips_types WHERE id = ? AND user_id = ?)
            ",
            id_param,
            type_id_param,
            id_param,
            user_id,
            type_id_param,
            user_id,
        )
        .await?;

        Ok(results.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn trip_type_add(
        ctx: &Context,
        pool: &sqlite::Pool,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<(), Error> {
        let user_id = ctx.user.id.to_string();
        // TODO user handling?
        let trip_id_param = id.to_string();
        let type_id_param = type_id.to_string();

        crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Insert,
                component: sqlite::Component::Trips,
            },
            pool,
            "INSERT INTO
                trips_to_trips_types (trip_id, trip_type_id)
            SELECT trips.id as trip_id, trips_types.id as trip_type_id
                FROM trips
                INNER JOIN trips_types
                WHERE
                    trips.id = ?
                    AND trips.user_id = ?
                    AND trips_types.id = ?
                    AND trips_types.user_id = ?",
            trip_id_param,
            user_id,
            type_id_param,
            user_id,
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn set_state(
        ctx: &Context,
        pool: &sqlite::Pool,
        id: Uuid,
        new_state: &TripState,
    ) -> Result<bool, Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = id.to_string();
        let result = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Update,
                component: sqlite::Component::Trips,
            },
            pool,
            "UPDATE trips
            SET state = ?
            WHERE id = ? and user_id = ?",
            new_state,
            trip_id_param,
            user_id,
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn set_comment(
        ctx: &Context,
        pool: &sqlite::Pool,
        id: Uuid,
        new_comment: &str,
    ) -> Result<bool, Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = id.to_string();
        let result = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Update,
                component: sqlite::Component::Trips,
            },
            pool,
            "UPDATE trips
            SET comment = ?
            WHERE id = ? AND user_id = ?",
            new_comment,
            trip_id_param,
            user_id,
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn set_attribute(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
        attribute: TripAttribute,
        value: &str,
    ) -> Result<(), Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = trip_id.to_string();
        let result = match attribute {
            TripAttribute::Name => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET name = ?
                WHERE id = ? AND user_id = ?",
                    value,
                    trip_id_param,
                    user_id
                )
                .await
            }

            TripAttribute::DateStart => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET date_start = ?
                WHERE id = ? AND user_id = ?",
                    value,
                    trip_id_param,
                    user_id
                )
                .await
            }
            TripAttribute::DateEnd => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET date_end = ?
                WHERE id = ? AND user_id = ?",
                    value,
                    trip_id_param,
                    user_id
                )
                .await
            }
            TripAttribute::Location => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET location = ?
                WHERE id = ? AND user_id = ?",
                    value,
                    trip_id_param,
                    user_id
                )
                .await
            }
            TripAttribute::TempMin => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET temp_min = ?
                WHERE id = ? AND user_id = ?",
                    value,
                    trip_id_param,
                    user_id
                )
                .await
            }
            TripAttribute::TempMax => {
                crate::execute!(
                    &sqlite::QueryClassification {
                        query_type: sqlite::QueryType::Update,
                        component: sqlite::Component::Trips,
                    },
                    pool,
                    "UPDATE trips
                SET temp_max = ?
                WHERE id = ? AND user_id = ?",
                    value,
                    trip_id_param,
                    user_id
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
        pool: &sqlite::Pool,
        name: &str,
        date_start: time::Date,
        date_end: time::Date,
        copy_from: Option<Uuid>,
    ) -> Result<Uuid, Error> {
        let user_id = ctx.user.id.to_string();
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let date_start = date_start.format(consts::DATE_FORMAT)?;
        let date_end = date_end.format(consts::DATE_FORMAT)?;

        let trip_state = TripState::new();

        let mut transaction = pool.begin().await?;

        crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Insert,
                component: sqlite::Component::Trips,
            },
            &mut *transaction,
            "INSERT INTO trips
                (id, name, date_start, date_end, state, user_id)
            VALUES
                (?, ?, ?, ?, ?, ?)",
            id_param,
            name,
            date_start,
            date_end,
            trip_state,
            user_id,
        )
        .await?;

        if let Some(copy_from_trip_id) = copy_from {
            let copy_from_trip_id_param = copy_from_trip_id.to_string();
            crate::execute!(
                &sqlite::QueryClassification {
                    query_type: sqlite::QueryType::Insert,
                    component: sqlite::Component::Trips,
                },
                &mut *transaction,
                r#"INSERT INTO trips_items (
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
                FROM trips_items
                WHERE trip_id = $2 AND user_id = $3"#,
                id_param,
                copy_from_trip_id_param,
                user_id,
            )
            .await?;
        } else {
            crate::execute!(
                &sqlite::QueryClassification {
                    query_type: sqlite::QueryType::Insert,
                    component: sqlite::Component::Trips,
                },
                &mut *transaction,
                r#"INSERT INTO trips_items (
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
                id_param,
                user_id,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(id)
    }

    #[tracing::instrument]
    pub async fn find_total_picked_weight(
        ctx: &Context,
        pool: &sqlite::Pool,
        trip_id: Uuid,
    ) -> Result<i64, Error> {
        let user_id = ctx.user.id.to_string();
        let trip_id_param = trip_id.to_string();
        let weight = crate::execute_returning!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
            },
            pool,
            "
                SELECT
                    CAST(IFNULL(SUM(i_item.weight), 0) AS INTEGER) AS total_weight
                FROM trips AS trip
                INNER JOIN trips_items AS t_item
                    ON t_item.trip_id = trip.id
                INNER JOIN inventory_items AS i_item
                    ON t_item.item_id = i_item.id
                WHERE
                    trip.id = ? AND trip.user_id = ?
                AND t_item.pick = true
            ",
            i64,
            |row| i64::from(row.total_weight),
            trip_id_param,
            user_id,
        )
        .await?;

        Ok(weight)
    }

    #[tracing::instrument]
    pub fn types(&self) -> &Vec<TripType> {
        self.types
            .as_ref()
            .expect("you need to call load_trips_types()")
    }

    #[tracing::instrument]
    pub fn categories(&self) -> &Vec<TripCategory> {
        self.categories
            .as_ref()
            .expect("you need to call load_trips_types()")
    }

    #[tracing::instrument]
    pub fn total_picked_weight(&self) -> i64 {
        self.categories()
            .iter()
            .map(|category| -> i64 {
                category
                    .items
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter_map(|item| Some(item.item.weight).filter(|_| item.picked))
                    .sum::<i64>()
            })
            .sum::<i64>()
    }

    #[tracing::instrument]
    pub async fn load_trips_types(
        &mut self,
        ctx: &Context,
        pool: &sqlite::Pool,
    ) -> Result<(), Error> {
        struct Row {
            id: String,
            name: String,
            active: i32,
        }

        impl TryFrom<Row> for TripType {
            type Error = Error;

            fn try_from(row: Row) -> Result<Self, Self::Error> {
                Ok(TripType {
                    id: Uuid::try_parse(&row.id)?,
                    name: row.name,
                    active: match row.active {
                        0 => false,
                        1 => true,
                        _ => unreachable!(),
                    },
                })
            }
        }

        let user_id = ctx.user.id.to_string();
        let id = self.id.to_string();
        let types = crate::query_all!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
            },
            pool,
            Row,
            TripType,
            "
            SELECT
                type.id as id,
                type.name as name,
                inner.id IS NOT NULL AS active
            FROM trips_types AS type
                LEFT JOIN (
                    SELECT type.id as id, trip.user_id as user_id
                    FROM trips as trip
                    INNER JOIN trips_to_trips_types as ttt
                        ON ttt.trip_id = trip.id
                    INNER JOIN trips_types AS type
                        ON type.id == ttt.trip_type_id
                    WHERE trip.id = ? AND trip.user_id = ?
                ) AS inner
                ON inner.id = type.id
            WHERE type.user_id = ?
            ",
            id,
            user_id,
            user_id,
        )
        .await?;

        self.types = Some(types);
        Ok(())
    }

    #[tracing::instrument]
    pub async fn sync_trip_items_with_inventory(
        &mut self,
        ctx: &Context,
        pool: &sqlite::Pool,
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
        let user_id = ctx.user.id.to_string();
        let trip_id = self.id.to_string();

        struct Row {
            item_id: String,
        }

        impl TryFrom<Row> for Uuid {
            type Error = Error;

            fn try_from(value: Row) -> Result<Self, Self::Error> {
                Uuid::try_parse(&value.item_id).map_err(Into::into)
            }
        }

        let unsynced_items: Vec<Uuid> = crate::query_all!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
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
                    FROM trips_items AS t_item
                    WHERE t_item.trip_id = ? AND t_item.user_id = ?
                ) AS t_item
                ON t_item.item_id = i_item.id
            WHERE t_item.item_id IS NULL AND i_item.user_id = ?",
            trip_id,
            user_id,
            user_id,
        )
        .await?;

        // only mark as new when the trip is already underway
        let mark_as_new = self.state != TripState::new();

        for unsynced_item in &unsynced_items {
            let item_id = unsynced_item.to_string();
            crate::execute!(
                &sqlite::QueryClassification {
                    query_type: sqlite::QueryType::Insert,
                    component: sqlite::Component::Trips,
                },
                pool,
                "
                INSERT INTO trips_items
                    (
                        item_id,
                        trip_id,
                        pick,
                        pack,
                        ready,
                        new,
                        user_id
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                ",
                item_id,
                trip_id,
                false,
                false,
                false,
                mark_as_new,
                user_id,
            )
            .await?;
        }

        tracing::info!("unsynced items: {:?}", &unsynced_items);

        Ok(())
    }

    #[tracing::instrument]
    pub async fn load_categories(
        &mut self,
        ctx: &Context,
        pool: &sqlite::Pool,
    ) -> Result<(), Error> {
        let mut categories: Vec<TripCategory> = vec![];
        // we can ignore the return type as we collect into `categories`
        // in the `map_ok()` closure
        let id = self.id.to_string();
        let user_id = ctx.user.id.to_string();

        struct Row {
            category_id: String,
            category_name: String,
            category_description: Option<String>,
            #[allow(dead_code)]
            trip_id: Option<String>,
            item_id: Option<String>,
            item_name: Option<String>,
            item_description: Option<String>,
            item_weight: Option<i64>,
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
                    id: Uuid::try_parse(&row.category_id)?,
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
                                id: Uuid::try_parse(&item_id)?,
                                name: row.item_name.unwrap(),
                                description: row.item_description,
                                weight: row.item_weight.unwrap(),
                                category_id: Uuid::try_parse(&row.category_id)?,
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
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
            },
            pool,
            Row,
            RowParsed,
            r#"
                SELECT
                    category.id as category_id,
                    category.name as category_name,
                    category.description AS category_description,
                    inner.trip_id AS trip_id,
                    inner.item_id AS item_id,
                    inner.item_name AS item_name,
                    inner.item_description AS item_description,
                    inner.item_weight AS item_weight,
                    inner.item_is_picked AS item_is_picked,
                    inner.item_is_packed AS item_is_packed,
                    inner.item_is_ready AS item_is_ready,
                    inner.item_is_new AS item_is_new
                FROM inventory_items_categories AS category
                    LEFT JOIN (
                        SELECT
                            trip.trip_id AS trip_id,
                            category.id as category_id,
                            category.name as category_name,
                            category.description as category_description,
                            item.id as item_id,
                            item.name as item_name,
                            item.description as item_description,
                            item.weight as item_weight,
                            trip.pick as item_is_picked,
                            trip.pack as item_is_packed,
                            trip.ready as item_is_ready,
                            trip.new as item_is_new,
                            trip.user_id as user_id
                        FROM trips_items as trip
                        INNER JOIN inventory_items as item
                            ON item.id = trip.item_id
                        INNER JOIN inventory_items_categories as category
                            ON category.id = item.category_id
                        WHERE trip.trip_id = ? AND trip.user_id = ?
                    ) AS inner
                    ON inner.category_id = category.id
                WHERE category.user_id = ?
            "#,
            id,
            user_id,
            user_id,
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

impl TripsType {
    #[tracing::instrument]
    pub async fn all(ctx: &Context, pool: &sqlite::Pool) -> Result<Vec<Self>, Error> {
        let user_id = ctx.user.id.to_string();
        crate::query_all!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::Trips,
            },
            pool,
            DbTripsTypesRow,
            Self,
            "SELECT
                id,
                name
            FROM trips_types
            WHERE user_id = ?",
            user_id,
        )
        .await
    }

    #[tracing::instrument]
    pub async fn save(ctx: &Context, pool: &sqlite::Pool, name: &str) -> Result<Uuid, Error> {
        let user_id = ctx.user.id.to_string();
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Insert,
                component: sqlite::Component::Trips,
            },
            pool,
            "INSERT INTO trips_types
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
    pub async fn set_name(
        ctx: &Context,
        pool: &sqlite::Pool,
        id: Uuid,
        new_name: &str,
    ) -> Result<bool, Error> {
        let user_id = ctx.user.id.to_string();
        let id_param = id.to_string();

        let result = crate::execute!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Update,
                component: sqlite::Component::Trips,
            },
            pool,
            "UPDATE trips_types
            SET name = ?
            WHERE id = ? and user_id = ?",
            new_name,
            id_param,
            user_id,
        )
        .await?;

        Ok(result.rows_affected() != 0)
    }
}

pub(crate) struct DbTripsTypesRow {
    pub id: String,
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
            id: Uuid::try_parse(&row.id)?,
            name: row.name,
        })
    }
}
