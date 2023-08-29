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

use sqlx::sqlite::SqlitePoolOptions;

use futures::TryFutureExt;
use futures::TryStreamExt;

use time::{
    error::Parse as TimeParseError, format_description::FormatItem, macros::format_description,
};

pub const DATE_FORMAT: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

pub enum Error {
    SqlError { description: String },
    UuidError { description: String },
    EnumError { description: String },
    NotFoundError { description: String },
    IntError { description: String },
    TimeParseError { description: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SqlError { description } => {
                write!(f, "SQL error: {description}")
            }
            Self::UuidError { description } => {
                write!(f, "UUID error: {description}")
            }
            Self::NotFoundError { description } => {
                write!(f, "Not found: {description}")
            }
            Self::IntError { description } => {
                write!(f, "Integer error: {description}")
            }
            Self::EnumError { description } => {
                write!(f, "Enum error: {description}")
            }
            Self::TimeParseError { description } => {
                write!(f, "Date parse error: {description}")
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
        Error::UuidError {
            description: value.to_string(),
        }
    }
}

impl convert::From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::SqlError {
            description: value.to_string(),
        }
    }
}

impl convert::From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Error::IntError {
            description: value.to_string(),
        }
    }
}

impl convert::From<TimeParseError> for Error {
    fn from(value: TimeParseError) -> Self {
        Error::TimeParseError {
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

    pub fn is_first(&self) -> bool {
        self == &TripState::new()
    }

    pub fn is_last(&self) -> bool {
        self == &TripState::Done
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
                return Err(Error::EnumError {
                    description: format!("{} is not a valid value for TripState", value),
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
}

#[derive(Debug)]
pub struct TripItem {
    pub item: Item,
    pub picked: bool,
    pub packed: bool,
    pub new: bool,
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

// impl std::convert::Into<&'static str> for TripAttribute {
//     fn into(self) -> &'static str {
//         match self {
//             Self::DateStart => "date_start",
//             Self::DateEnd => "date_end",
//             Self::Location => "location",
//             Self::TempMin => "temp_min",
//             Self::TempMax => "temp_max",
//         }
//     }
// }

// impl std::convert::TryFrom<&str> for TripAttribute {
//     type Error = Error;

//     fn try_from(value: &str) -> Result<Self, Error> {
//         Ok(match value {
//             "date_start" => Self::DateStart,
//             "date_end" => Self::DateEnd,
//             "location" => Self::Location,
//             "temp_min" => Self::TempMin,
//             "temp_max" => Self::TempMax,
//             _ => {
//                 return Err(Error::UnknownAttributeValue {
//                     attribute: value.to_string(),
//                 })
//             }
//         })
//     }
// }

// impl TryFrom<SqliteRow> for Trip {
//     type Error = Error;

//     fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
//         let name: &str = row.try_get("name")?;
//         let id: &str = row.try_get("id")?;
//         let date_start: time::Date = row.try_get("date_start")?;
//         let date_end: time::Date = row.try_get("date_end")?;
//         let state: TripState = row.try_get("state")?;
//         let location = row.try_get("location")?;
//         let temp_min = row.try_get("temp_min")?;
//         let temp_max = row.try_get("temp_max")?;
//         let comment = row.try_get("comment")?;

//         let id: Uuid = Uuid::try_parse(id)?;

//         Ok(Trip {
//             id,
//             name: name.to_string(),
//             date_start,
//             date_end,
//             state,
//             location,
//             temp_min,
//             temp_max,
//             comment,
//             types: None,
//             categories: None,
//         })
//     }
// }

impl<'a> Trip {
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
}

impl<'a> Trip {
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

// impl TryFrom<SqliteRow> for TripType {
//     type Error = Error;

//     fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
//         let id: Uuid = Uuid::try_parse(row.try_get("id")?)?;
//         let name: String = row.try_get::<&str, _>("name")?.to_string();
//         let active: bool = row.try_get("active")?;

//         Ok(Self { id, name, active })
//     }
// }

pub struct DbCategoryRow {
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

// impl TryFrom<SqliteRow> for Category {
//     type Error = Error;

//     fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
//         let name: &str = row.try_get("name")?;
//         let description: &str = row.try_get("description")?;
//         let id: Uuid = Uuid::try_parse(row.try_get("id")?)?;

//         Ok(Category {
//             id,
//             name: name.to_string(),
//             description: description.to_string(),
//             items: None,
//         })
//     }
// }

pub struct DbInventoryItemsRow {
    id: String,
    name: String,
    weight: i64,
    description: Option<String>,
    category_id: String,
}

impl<'a> Category {
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

// impl TryFrom<SqliteRow> for Item {
//     type Error = Error;

//     fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
//         let name: &str = row.try_get("name")?;
//         let description: &str = row.try_get("description")?;
//         let weight: i64 = row.try_get("weight")?;
//         let id: Uuid = Uuid::try_parse(row.try_get("id")?)?;
//         let category_id: Uuid = Uuid::try_parse(row.try_get("category_id")?)?;

//         Ok(Item {
//             id,
//             name: name.to_string(),
//             weight,
//             description: description.to_string(),
//             category_id,
//         })
//     }
// }

impl Item {
    pub async fn find(pool: &sqlx::Pool<sqlx::Sqlite>, id: Uuid) -> Result<Option<Item>, Error> {
        let id_param = id.to_string();
        let item: Result<Result<Item, Error>, sqlx::Error> = sqlx::query_as!(
            DbInventoryItemsRow,
            "SELECT * FROM inventory_items AS item
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
}
