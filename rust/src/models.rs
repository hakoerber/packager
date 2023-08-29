use serde::{Deserialize, Serialize};
use sqlx::{
    database::Database,
    database::HasValueRef,
    sqlite::{Sqlite, SqliteRow},
    Decode, Row,
};
use std::convert;
use std::error;
use std::fmt;
use std::num::TryFromIntError;
use std::str::FromStr;
use uuid::Uuid;

use sqlx::{error::DatabaseError, sqlite::SqlitePoolOptions};

use futures::TryFutureExt;
use futures::TryStreamExt;

use time::{error::Parse as TimeParse, format_description::FormatItem, macros::format_description};

pub const DATE_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

pub enum Error {
    Sql { description: String },
    Uuid { description: String },
    Enum { description: String },
    Int { description: String },
    Constraint { description: String },
    TimeParse { description: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sql { description } => {
                write!(f, "SQL error: {description}")
            }
            Self::Uuid { description } => {
                write!(f, "UUID error: {description}")
            }
            Self::Int { description } => {
                write!(f, "Integer error: {description}")
            }
            Self::Enum { description } => {
                write!(f, "Enum error: {description}")
            }
            Self::TimeParse { description } => {
                write!(f, "Date parse error: {description}")
            }
            Self::Constraint { description } => {
                write!(f, "SQL constraint error: {description}")
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // defer to Display
        write!(f, "SQL error: {self}")
    }
}

impl convert::From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Error::Uuid {
            description: value.to_string(),
        }
    }
}

impl convert::From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Sql {
            description: value.to_string(),
        }
    }
}

impl convert::From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Error::Int {
            description: value.to_string(),
        }
    }
}

impl convert::From<TimeParse> for Error {
    fn from(value: TimeParse) -> Self {
        Error::TimeParse {
            description: value.to_string(),
        }
    }
}

impl error::Error for Error {}

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
                return Err(Error::Enum {
                    description: format!("{value} is not a valid value for TripState"),
                })
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
    pub category: Category,
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
                        category: Category {
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
                        item: Item {
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
    pub item: Item,
    pub picked: bool,
    pub packed: bool,
    pub new: bool,
}

struct DbTripsItemsRow {
    picked: bool,
    packed: bool,
    new: bool,
    id: String,
    name: String,
    weight: i64,
    description: Option<String>,
    category_id: String,
}

impl TryFrom<DbTripsItemsRow> for TripItem {
    type Error = Error;

    fn try_from(row: DbTripsItemsRow) -> Result<Self, Self::Error> {
        Ok(TripItem {
            picked: row.picked,
            packed: row.packed,
            new: row.new,
            item: Item {
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
        let item: Result<Result<TripItem, Error>, sqlx::Error> = sqlx::query_as!(
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
        .fetch_one(pool)
        .map_ok(|row| row.try_into())
        .await;

        match item {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
            Ok(v) => Ok(Some(v?)),
        }
    }
}

struct DbTripRow {
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
            date_start: time::Date::parse(&row.date_start, DATE_FORMAT)?,
            date_end: time::Date::parse(&row.date_end, DATE_FORMAT)?,
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
    types: Option<Vec<TripType>>,
    categories: Option<Vec<TripCategory>>,
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

struct DbTripWeightRow {
    pub total_weight: Option<i32>,
}

impl<'a> Trip {
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
        let trip = sqlx::query_as!(
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
        .fetch_one(pool)
        .map_ok(|row| row.try_into())
        .await;

        match trip {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
            Ok(v) => Ok(Some(v?)),
        }
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

    pub async fn save(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
        date_start: time::Date,
        date_end: time::Date,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let date_start = date_start
            .format(DATE_FORMAT)
            .map_err(|e| Error::TimeParse {
                description: e.to_string(),
            })?;
        let date_end = date_end.format(DATE_FORMAT).map_err(|e| Error::TimeParse {
            description: e.to_string(),
        })?;

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
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref error) => {
                let sqlite_error = error.downcast_ref::<sqlx::sqlite::SqliteError>();
                if let Some(code) = sqlite_error.code() {
                    match &*code {
                        // SQLITE_CONSTRAINT_FOREIGNKEY
                        "787" => Error::Constraint {
                            description: format!(
                                "SQLITE_CONSTRAINT_FOREIGNKEY on table without foreignkey?",
                            ),
                        },
                        // SQLITE_CONSTRAINT_UNIQUE
                        "2067" => Error::Constraint {
                            description: format!("trip with name \"{name}\" already exists",),
                        },
                        _ => Error::Sql {
                            description: format!(
                                "got error with unknown code: {}",
                                sqlite_error.to_string()
                            ),
                        },
                    }
                } else {
                    Error::Sql {
                        description: format!(
                            "got error without code: {}",
                            sqlite_error.to_string()
                        ),
                    }
                }
            }
            _ => Error::Sql {
                description: format!("got unknown error: {}", e.to_string()),
            },
        })?;

        Ok(id)
    }

    pub async fn find_total_picked_weight(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        trip_id: Uuid,
    ) -> Result<Option<i64>, Error> {
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
        .map_ok(|row| row.total_weight.map(|weight| weight as i64))
        .await;

        match weight {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
            Ok(v) => Ok(v),
        }
    }

    pub fn types(&'a self) -> &Vec<TripType> {
        self.types
            .as_ref()
            .expect("you need to call load_trips_types()")
    }

    pub fn categories(&'a self) -> &Vec<TripCategory> {
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

    pub async fn load_trips_types(
        &'a mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
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
        &'a mut self,
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

    pub async fn load_categories(
        &'a mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
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
                category: Category {
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
                        item: Item {
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

struct DbCategoryRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    items: Option<Vec<Item>>,
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

struct DbInventoryItemsRow {
    id: String,
    name: String,
    weight: i64,
    description: Option<String>,
    category_id: String,
}

impl<'a> Category {
    pub async fn _find(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
    ) -> Result<Option<Category>, Error> {
        let id_param = id.to_string();
        let item: Result<Result<Category, Error>, sqlx::Error> = sqlx::query_as!(
            DbCategoryRow,
            "SELECT
                id,
                name,
                description
            FROM inventory_items_categories AS category
            WHERE category.id = ?",
            id_param,
        )
        .fetch_one(pool)
        .map_ok(|row| row.try_into())
        .await;

        match item {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
            Ok(v) => Ok(Some(v?)),
        }
    }

    pub fn items(&'a self) -> &'a Vec<Item> {
        self.items
            .as_ref()
            .expect("you need to call populate_items()")
    }

    pub fn total_weight(&self) -> i64 {
        self.items().iter().map(|item| item.weight).sum()
    }

    pub async fn populate_items(
        &'a mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
        let id = self.id.to_string();
        let items = sqlx::query_as!(
            DbInventoryItemsRow,
            "SELECT
                id,
                name,
                weight,
                description,
                category_id
            FROM inventory_items
            WHERE category_id = ?",
            id
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
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub weight: i64,
    pub category_id: Uuid,
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
    pub async fn find(pool: &sqlx::Pool<sqlx::Sqlite>, id: Uuid) -> Result<Option<Item>, Error> {
        let id_param = id.to_string();
        let item: Result<Result<Item, Error>, sqlx::Error> = sqlx::query_as!(
            DbInventoryItemsRow,
            "SELECT
                id,
                name,
                weight,
                description,
                category_id
            FROM inventory_items AS item
            WHERE item.id = ?",
            id_param,
        )
        .fetch_one(pool)
        .map_ok(|row| row.try_into())
        .await;

        match item {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
            Ok(v) => Ok(Some(v?)),
        }
    }

    pub async fn update(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        id: Uuid,
        name: &str,
        weight: i64,
    ) -> Result<Option<Uuid>, Error> {
        let id_param = id.to_string();
        let id: Result<Result<Uuid, Error>, sqlx::Error> = sqlx::query!(
            "UPDATE inventory_items AS item
            SET
                name = ?,
                weight = ?
            WHERE item.id = ?
            RETURNING inventory_items.category_id AS id
            ",
            name,
            weight,
            id_param,
        )
        .fetch_one(pool)
        .map_ok(|row| {
            let id: &str = &row.id.unwrap(); // TODO
            let uuid: Result<Uuid, uuid::Error> = Uuid::try_parse(id);
            let uuid: Result<Uuid, Error> = uuid.map_err(|e| e.into());
            uuid
        })
        .await;

        match id {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
            Ok(v) => Ok(Some(v?)),
        }
    }

    pub async fn get_category_max_weight(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        category_id: Uuid,
    ) -> Result<i64, Error> {
        let category_id_param = category_id.to_string();
        let weight: Result<i64, sqlx::Error> = sqlx::query!(
            "
                SELECT COALESCE(MAX(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                WHERE category_id = ?
            ",
            category_id_param
        )
        .fetch_one(pool)
        .map_ok(|row| {
            // convert to i64 because that the default integer type, but looks
            // like COALESCE return i32?
            //
            // We can be certain that the row exists, as we COALESCE it
            row.weight.unwrap() as i64
        })
        .await;

        Ok(weight?)
    }

    pub async fn _get_category_total_picked_weight(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        category_id: Uuid,
    ) -> Result<i64, Error> {
        let category_id_param = category_id.to_string();
        let weight: Result<i64, sqlx::Error> = sqlx::query!(
            "
                SELECT COALESCE(SUM(i_item.weight), 0) as weight
                FROM inventory_items_categories as category
                INNER JOIN inventory_items as i_item
                    ON i_item.category_id = category.id
                INNER JOIN trips_items as t_item
                    ON i_item.id = t_item.item_id
                WHERE category_id = ?
                AND t_item.pick = 1
            ",
            category_id_param
        )
        .fetch_one(pool)
        .map_ok(|row| {
            // convert to i64 because that the default integer type, but looks
            // like COALESCE return i32?
            //
            // We can be certain that the row exists, as we COALESCE it
            row.weight.unwrap() as i64
        })
        .await;

        Ok(weight?)
    }
}

struct DbTripsTypesRow {
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

pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub comment: Option<String>,
}

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
    pub async fn find(pool: &sqlx::Pool<sqlx::Sqlite>, id: Uuid) -> Result<Option<Self>, Error> {
        let id_param = id.to_string();

        let item: Result<Result<InventoryItem, Error>, sqlx::Error> = sqlx::query_as!(
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
                WHERE item.id = ?",
            id_param,
        )
        .fetch_one(pool)
        .map_ok(|row| row.try_into())
        .await;

        match item {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into()),
            },
            Ok(v) => Ok(Some(v?)),
        }
    }

    pub async fn name_exists(pool: &sqlx::Pool<sqlx::Sqlite>, name: &str) -> Result<bool, Error> {
        let item: Result<(), sqlx::Error> = sqlx::query!(
            "SELECT id
            FROM inventory_items
            WHERE name = ?",
            name,
        )
        .fetch_one(pool)
        .map_ok(|_row| ())
        .await;

        match item {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(false),
                _ => Err(e.into()),
            },
            Ok(_) => Ok(true),
        }
    }

    pub async fn delete(pool: &sqlx::Pool<sqlx::Sqlite>, id: Uuid) -> Result<bool, Error> {
        let id_param = id.to_string();
        let results = sqlx::query!(
            "DELETE FROM inventory_items
            WHERE id = ?",
            id_param
        )
        .execute(pool)
        .map_err(|error| match error {
            sqlx::Error::Database(ref error) => {
                let sqlite_error = error.downcast_ref::<sqlx::sqlite::SqliteError>();
                if let Some(code) = sqlite_error.code() {
                    match &*code {
                        "787" => {
                            // SQLITE_CONSTRAINT_FOREIGNKEY
                            Error::Constraint { description: format!("item {} cannot be deleted because it's on use in trips. instead, archive it", code.to_string()) }
                        }
                        _ =>
                            Error::Sql { description: format!("got error with unknown code: {}", sqlite_error.to_string()) }
                    }
                } else {
                    Error::Constraint { description: format!("got error without code: {}", sqlite_error.to_string()) }
                }
            }
            _ => Error::Constraint { description: format!("got unknown error: {}", error.to_string()) }
        }).await?;

        Ok(results.rows_affected() != 0)
    }

    pub async fn save(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
        category_id: Uuid,
        weight: u32,
    ) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        let id_param = id.to_string();
        let category_id_param = category_id.to_string();

        sqlx::query!(
            "INSERT INTO inventory_items
                (id, name, description, weight, category_id)
            VALUES
                (?, ?, ?, ?, ?)",
            id_param,
            name,
            "",
            weight,
            category_id_param
        )
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref error) => {
                let sqlite_error = error.downcast_ref::<sqlx::sqlite::SqliteError>();
                if let Some(code) = sqlite_error.code() {
                    match &*code {
                        // SQLITE_CONSTRAINT_FOREIGNKEY
                        "787" => Error::Constraint {
                            description: format!(
                                "category {category_id} not found",
                            ),
                        },
                        // SQLITE_CONSTRAINT_UNIQUE
                        "2067" => Error::Constraint {
                            description: format!(
                                "item with name \"{name}\" already exists in category {category_id}",
                            ),
                        },
                        _ => Error::Sql {
                            description: format!(
                                "got error with unknown code: {}",
                                sqlite_error.to_string()
                            ),
                        },
                    }
                } else {
                    Error::Sql {
                        description: format!(
                            "got error without code: {}",
                            sqlite_error.to_string()
                        ),
                    }
                }
            }
            _ => Error::Sql {
                description: format!("got unknown error: {}", e.to_string()),
            },
        })?;

        Ok(id)
    }
}

pub struct Inventory {
    pub categories: Vec<Category>,
}

impl Inventory {
    pub async fn load(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Self, Error> {
        let mut categories = sqlx::query_as!(
            DbCategoryRow,
            "SELECT id,name,description FROM inventory_items_categories"
        )
        .fetch(pool)
        .map_ok(|row: DbCategoryRow| row.try_into())
        .try_collect::<Vec<Result<Category, Error>>>()
        .await
        // we have two error handling lines here. these are distinct errors
        // this one is the SQL error that may arise during the query
        .map_err(|e| Error::Sql {
            description: e.to_string(),
        })?
        .into_iter()
        .collect::<Result<Vec<Category>, Error>>()
        // and this one is the model mapping error that may arise e.g. during
        // reading of the rows
        .map_err(|e| Error::Sql {
            description: e.to_string(),
        })?;

        for category in &mut categories {
            category.populate_items(pool).await?;
        }

        Ok(Self { categories })
    }
}
