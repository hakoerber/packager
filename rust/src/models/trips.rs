use std::fmt;

use super::{
    consts,
    error::{DatabaseError, Error, QueryError},
    inventory,
};

use futures::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;
use time;
use uuid::Uuid;

#[derive(sqlx::Type, PartialEq, PartialOrd, Deserialize)]
pub enum TripState {
    Init,
    Planning,
    Planned,
    Active,
    Review,
    Done,
}

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
}

impl fmt::Display for TripItemStateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pick => "pick",
                Self::Pack => "pack",
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
    pub fn total_picked_weight(&self) -> i64 {
        self.items
            .as_ref()
            .unwrap()
            .iter()
            .filter(|item| item.picked)
            .map(|item| item.item.weight)
            .sum()
    }

    pub async fn find(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        trip_id: Uuid,
        category_id: Uuid,
    ) -> Result<Option<TripCategory>, Error> {
        let mut category: Option<TripCategory> = None;

        let trip_id_param = trip_id.to_string();
        let category_id_param = category_id.to_string();

        sqlx::query!(
            "
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
                            trip.new as item_is_new
                        FROM trips_items as trip
                        INNER JOIN inventory_items as item
                            ON item.id = trip.item_id
                        INNER JOIN inventory_items_categories as category
                            ON category.id = item.category_id
                        WHERE trip.trip_id = ?
                    ) AS inner
                    ON inner.category_id = category.id
                WHERE category.id = ?
            ",
            trip_id_param,
            category_id_param
        )
        .fetch(pool)
        .map_ok(|row| -> Result<(), Error> {
            match &category {
                Some(_) => (),
                None => {
                    category = Some(TripCategory {
                        category: inventory::Category {
                            id: Uuid::try_parse(&row.category_id)?,
                            name: row.category_name,
                            description: row.category_description,
                            items: None,
                        },
                        items: None,
                    })
                }
            };

            match row.item_id {
                None => {
                    // we have an empty (unused) category which has NULL values
                    // for the item_id column
                    category.as_mut().unwrap().items = Some(vec![]);
                    category.as_mut().unwrap().category.items = Some(vec![]);
                }
                Some(item_id) => {
                    let item = TripItem {
                        item: inventory::Item {
                            id: Uuid::try_parse(&item_id)?,
                            name: row.item_name.unwrap(),
                            description: row.item_description,
                            weight: row.item_weight.unwrap(),
                            category_id: category.as_ref().unwrap().category.id,
                        },
                        picked: row.item_is_picked.unwrap(),
                        packed: row.item_is_packed.unwrap(),
                        new: row.item_is_new.unwrap(),
                    };

                    match &mut category.as_mut().unwrap().items {
                        None => category.as_mut().unwrap().items = Some(vec![item]),
                        Some(ref mut items) => items.push(item),
                    }
                }
            }

            Ok(())
        })
        .try_collect::<Vec<Result<(), Error>>>()
        .await?
        .into_iter()
        .collect::<Result<(), Error>>()?;

        // this may be None if there are no results (which
        // means that the category was not found)
        Ok(category)
    }
}

#[derive(Debug)]
pub struct TripItem {
    pub item: inventory::Item,
    pub picked: bool,
    pub packed: bool,
    pub new: bool,
}

pub(crate) struct DbTripsItemsRow {
    pub(crate) picked: bool,
    pub(crate) packed: bool,
    pub(crate) new: bool,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) weight: i64,
    pub(crate) description: Option<String>,
    pub(crate) category_id: String,
}

impl TryFrom<DbTripsItemsRow> for TripItem {
    type Error = Error;

    fn try_from(row: DbTripsItemsRow) -> Result<Self, Self::Error> {
        Ok(TripItem {
            picked: row.picked,
            packed: row.packed,
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
    pub async fn find(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        trip_id: Uuid,
        item_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let item_id_param = item_id.to_string();
        let trip_id_param = trip_id.to_string();
        sqlx::query_as!(
            DbTripsItemsRow,
            "
                SELECT
                    t_item.item_id AS id,
                    t_item.pick AS picked,
                    t_item.pack AS packed,
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
            ",
            item_id_param,
            trip_id_param,
        )
        .fetch_optional(pool)
        .await?
        .map(|row| row.try_into())
        .transpose()
    }

    pub async fn set_state(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        trip_id: Uuid,
        item_id: Uuid,
        key: TripItemStateKey,
        value: bool,
    ) -> Result<(), Error> {
        let result = sqlx::query(&format!(
            "UPDATE trips_items
            SET {key} = ?
            WHERE trip_id = ?
            AND item_id = ?",
            key = to_variant_name(&key).unwrap()
        ))
        .bind(value)
        .bind(trip_id.to_string())
        .bind(item_id.to_string())
        .execute(pool)
        .await?;

        (result.rows_affected() != 0).then_some(()).ok_or_else(|| {
            Error::Query(QueryError::NotFound {
                description: format!("item {item_id} not found for trip {trip_id}"),
            })
        })
    }
}

pub(crate) struct DbTripRow {
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

pub(crate) struct DbTripWeightRow {
    pub total_weight: Option<i32>,
}

impl Trip {
    pub async fn all(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<Trip>, Error> {
        sqlx::query_as!(
            DbTripRow,
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
            FROM trips",
        )
        .fetch(pool)
        .map_ok(|row| row.try_into())
        .try_collect::<Vec<Result<Trip, Error>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<Trip>, Error>>()
    }

    pub async fn find(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        trip_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        let trip_id_param = trip_id.to_string();
        sqlx::query_as!(
            DbTripRow,
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
            WHERE id = ?",
            trip_id_param
        )
        .fetch_optional(pool)
        .await?
        .map(|row| row.try_into())
        .transpose()
    }

    pub async fn trip_type_remove(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<bool, Error> {
        let id_param = id.to_string();
        let type_id_param = type_id.to_string();

        let results = sqlx::query!(
            "DELETE FROM trips_to_trips_types AS ttt
            WHERE ttt.trip_id = ?
                AND ttt.trip_type_id = ?
            ",
            id_param,
            type_id_param,
        )
        .execute(pool)
        .await?;

        Ok(results.rows_affected() != 0)
    }

    pub async fn trip_type_add(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
        type_id: Uuid,
    ) -> Result<(), Error> {
        let trip_id_param = id.to_string();
        let type_id_param = type_id.to_string();
        sqlx::query!(
            "INSERT INTO trips_to_trips_types
                (trip_id, trip_type_id)
            VALUES
                (?, ?)
            ",
            trip_id_param,
            type_id_param,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_state(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
        new_state: &TripState,
    ) -> Result<bool, Error> {
        let trip_id_param = id.to_string();
        let result = sqlx::query!(
            "UPDATE trips
            SET state = ?
            WHERE id = ?",
            new_state,
            trip_id_param,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn set_comment(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
        new_comment: &str,
    ) -> Result<bool, Error> {
        let trip_id_param = id.to_string();
        let result = sqlx::query!(
            "UPDATE trips
            SET comment = ?
            WHERE id = ?",
            new_comment,
            trip_id_param,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn set_attribute(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        trip_id: Uuid,
        attribute: TripAttribute,
        value: &str,
    ) -> Result<(), Error> {
        let result = sqlx::query(&format!(
            "UPDATE trips
            SET {attribute} = ?
            WHERE id = ?",
            attribute = to_variant_name(&attribute).unwrap()
        ))
        .bind(value)
        .bind(trip_id.to_string())
        .execute(pool)
        .await?;

        (result.rows_affected() != 0).then_some(()).ok_or_else(|| {
            Error::Query(QueryError::NotFound {
                description: format!("trip {trip_id} not found"),
            })
        })
    }

    pub async fn save(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
        date_start: time::Date,
        date_end: time::Date,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let date_start = date_start.format(consts::DATE_FORMAT)?;
        let date_end = date_end.format(consts::DATE_FORMAT)?;

        let trip_state = TripState::new();

        sqlx::query!(
            "INSERT INTO trips
                (id, name, date_start, date_end, state)
            VALUES
                (?, ?, ?, ?, ?)",
            id_param,
            name,
            date_start,
            date_end,
            trip_state,
        )
        .execute(pool)
        .await?;

        Ok(id)
    }

    pub async fn find_total_picked_weight(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        trip_id: Uuid,
    ) -> Result<i64, Error> {
        let trip_id_param = trip_id.to_string();
        let weight = sqlx::query_as!(
            DbTripWeightRow,
            "
                SELECT
                    CAST(IFNULL(SUM(i_item.weight), 0) AS INTEGER) AS total_weight
                FROM trips AS trip
                INNER JOIN trips_items AS t_item
                    ON t_item.trip_id = trip.id
                INNER JOIN inventory_items AS i_item
                    ON t_item.item_id = i_item.id
                WHERE
                    trip.id = ?
                AND t_item.pick = true
            ",
            trip_id_param
        )
        .fetch_one(pool)
        .map_ok(|row| row.total_weight.unwrap() as i64)
        .await?;

        Ok(weight)
    }

    pub fn types(&self) -> &Vec<TripType> {
        self.types
            .as_ref()
            .expect("you need to call load_trips_types()")
    }

    pub fn categories(&self) -> &Vec<TripCategory> {
        self.categories
            .as_ref()
            .expect("you need to call load_trips_types()")
    }

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

    pub async fn load_trips_types(&mut self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), Error> {
        let id = self.id.to_string();
        let types = sqlx::query!(
            "
            SELECT
                type.id as id,
                type.name as name,
                inner.id IS NOT NULL AS active
            FROM trips_types AS type
                LEFT JOIN (
                    SELECT type.id as id, type.name as name
                    FROM trips as trip
                    INNER JOIN trips_to_trips_types as ttt
                        ON ttt.trip_id = trip.id
                    INNER JOIN trips_types AS type
                        ON type.id == ttt.trip_type_id
                    WHERE trip.id = ?
                ) AS inner
                ON inner.id = type.id
            ",
            id
        )
        .fetch(pool)
        .map_ok(|row| -> Result<TripType, Error> {
            Ok(TripType {
                id: Uuid::try_parse(&row.id)?,
                name: row.name,
                active: match row.active {
                    0 => false,
                    1 => true,
                    _ => unreachable!(),
                },
            })
        })
        .try_collect::<Vec<Result<TripType, Error>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<TripType>, Error>>()?;

        self.types = Some(types);
        Ok(())
    }

    pub async fn sync_trip_items_with_inventory(
        &mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
        // we need to get all items that are part of the inventory but not
        // part of the trip items
        //
        // then, we know which items we need to sync. there are different
        // states for them:
        //
        // * if the trip is new (it's state is INITIAL), we can just forward
        //   as-is
        // * if the trip is new, we have to make these new items prominently
        //   visible so the user knows that there might be new items to
        //   consider
        let trip_id = self.id.to_string();
        let unsynced_items: Vec<Uuid> = sqlx::query!(
            "
            SELECT
                i_item.id AS item_id
            FROM inventory_items AS i_item
                LEFT JOIN (
                    SELECT t_item.item_id as item_id
                    FROM trips_items AS t_item
                    WHERE t_item.trip_id = ?
                ) AS t_item
                ON t_item.item_id = i_item.id
            WHERE t_item.item_id IS NULL
        ",
            trip_id
        )
        .fetch(pool)
        .map_ok(|row| -> Result<Uuid, Error> { Ok(Uuid::try_parse(&row.item_id)?) })
        .try_collect::<Vec<Result<Uuid, Error>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<Uuid>, Error>>()?;

        // looks like there is currently no nice way to do multiple inserts
        // with sqlx. whatever, this won't matter

        // only mark as new when the trip is already underway
        let mark_as_new = self.state != TripState::new();

        for unsynced_item in &unsynced_items {
            let item_id = unsynced_item.to_string();
            sqlx::query!(
                "
                INSERT INTO trips_items
                    (
                        item_id,
                        trip_id,
                        pick,
                        pack,
                        new
                    )
                    VALUES (?, ?, ?, ?, ?)
                ",
                item_id,
                trip_id,
                false,
                false,
                mark_as_new,
            )
            .execute(pool)
            .await?;
        }

        tracing::info!("unsynced items: {:?}", &unsynced_items);

        Ok(())
    }

    pub async fn load_categories(&mut self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), Error> {
        let mut categories: Vec<TripCategory> = vec![];
        // we can ignore the return type as we collect into `categories`
        // in the `map_ok()` closure
        let id = self.id.to_string();
        sqlx::query!(
            "
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
                            trip.new as item_is_new
                        FROM trips_items as trip
                        INNER JOIN inventory_items as item
                            ON item.id = trip.item_id
                        INNER JOIN inventory_items_categories as category
                            ON category.id = item.category_id
                        WHERE trip.trip_id = ?
                    ) AS inner
                    ON inner.category_id = category.id
            ",
            id
        )
        .fetch(pool)
        .map_ok(|row| -> Result<(), Error> {
            let mut category = TripCategory {
                category: inventory::Category {
                    id: Uuid::try_parse(&row.category_id)?,
                    name: row.category_name,
                    description: row.category_description,

                    items: None,
                },
                items: None,
            };

            match row.item_id {
                None => {
                    // we have an empty (unused) category which has NULL values
                    // for the item_id column
                    category.items = Some(vec![]);
                    categories.push(category);
                }
                Some(item_id) => {
                    let item = TripItem {
                        item: inventory::Item {
                            id: Uuid::try_parse(&item_id)?,
                            name: row.item_name.unwrap(),
                            description: row.item_description,
                            weight: row.item_weight.unwrap(),
                            category_id: category.category.id,
                        },
                        picked: row.item_is_picked.unwrap(),
                        packed: row.item_is_packed.unwrap(),
                        new: row.item_is_new.unwrap(),
                    };

                    if let Some(&mut ref mut c) = categories
                        .iter_mut()
                        .find(|c| c.category.id == category.category.id)
                    {
                        // we always populate c.items when we add a new category, so
                        // it's safe to unwrap here
                        c.items.as_mut().unwrap().push(item);
                    } else {
                        category.items = Some(vec![item]);
                        categories.push(category);
                    }
                }
            }

            Ok(())
        })
        .try_collect::<Vec<Result<(), Error>>>()
        .await?
        .into_iter()
        .collect::<Result<(), Error>>()?;

        self.categories = Some(categories);

        Ok(())
    }
}

pub struct TripType {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
}

impl TripsType {
    pub async fn all(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<Self>, Error> {
        sqlx::query_as!(
            DbTripsTypesRow,
            "SELECT
                id,
                name
            FROM trips_types",
        )
        .fetch(pool)
        .map_ok(|row| row.try_into())
        .try_collect::<Vec<Result<Self, Error>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<Self>, Error>>()
    }

    pub async fn save(pool: &sqlx::Pool<sqlx::Sqlite>, name: &str) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        sqlx::query!(
            "INSERT INTO trips_types
            (id, name)
        VALUES
            (?, ?)",
            id_param,
            name,
        )
        .execute(pool)
        .await?;

        Ok(id)
    }

    pub async fn set_name(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
        new_name: &str,
    ) -> Result<bool, Error> {
        let id_param = id.to_string();

        let result = sqlx::query!(
            "UPDATE trips_types
            SET name = ?
            WHERE id = ?",
            new_name,
            id_param,
        )
        .execute(pool)
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
