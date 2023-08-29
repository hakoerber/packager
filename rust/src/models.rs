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
use std::str::FromStr;
use uuid::Uuid;

use sqlx::sqlite::SqlitePoolOptions;

use futures::TryFutureExt;
use futures::TryStreamExt;

pub enum Error {
    SqlError { description: String },
    UuidError { description: String },
    NotFoundError { description: String },
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

impl error::Error for Error {}

#[derive(sqlx::Type)]
pub enum TripState {
    Planning,
    Planned,
    Active,
    Review,
    Done,
}

impl fmt::Display for TripState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Planning => "Planning",
                Self::Planned => "Planned",
                Self::Active => "Active",
                Self::Review => "Review",
                Self::Done => "Done",
            },
        )
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
    pub fn total_picked_weight(&self) -> u32 {
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
}

pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub date_start: time::Date,
    pub date_end: time::Date,
    pub state: TripState,
    pub location: String,
    pub temp_min: i32,
    pub temp_max: i32,
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

impl TryFrom<SqliteRow> for Trip {
    type Error = Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let name: &str = row.try_get("name")?;
        let id: &str = row.try_get("id")?;
        let date_start: time::Date = row.try_get("date_start")?;
        let date_end: time::Date = row.try_get("date_end")?;
        let state: TripState = row.try_get("state")?;
        let location = row.try_get("location")?;
        let temp_min = row.try_get("temp_min")?;
        let temp_max = row.try_get("temp_max")?;
        let comment = row.try_get("comment")?;

        let id: Uuid = Uuid::try_parse(id)?;

        Ok(Trip {
            id,
            name: name.to_string(),
            date_start,
            date_end,
            state,
            location,
            temp_min,
            temp_max,
            comment,
            types: None,
            categories: None,
        })
    }
}

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
    pub async fn load_trips_types(
        &'a mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
        let types = sqlx::query(
            "
            SELECT
                type.id as id,
                type.name as name,
                CASE WHEN inner.id IS NOT NULL THEN true ELSE false END AS active
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
        )
        .bind(self.id.to_string())
        .fetch(pool)
        .map_ok(std::convert::TryInto::try_into)
        .try_collect::<Vec<Result<TripType, Error>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<TripType>, Error>>()?;

        self.types = Some(types);
        Ok(())
    }

    pub async fn load_categories(
        &'a mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
        let mut categories: Vec<TripCategory> = vec![];
        // we can ignore the return type as we collect into `categories`
        // in the `map_ok()` closure
        sqlx::query(
            "
                SELECT
                    category.id as category_id,
                    category.name as category_name,
                    category.description AS category_description,
                    inner.trip_id AS trip_id,
                    inner.category_description AS category_description,
                    inner.item_id AS item_id,
                    inner.item_name AS item_name,
                    inner.item_description AS item_description,
                    inner.item_weight AS item_weight,
                    inner.item_is_picked AS item_is_picked,
                    inner.item_is_packed AS item_is_packed
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
                            trip.pack as item_is_packed
                        FROM trips_items as trip
                        INNER JOIN inventory_items as item
                            ON item.id = trip.item_id
                        INNER JOIN inventory_items_categories as category
                            ON category.id = item.category_id
                        WHERE trip.trip_id = 'a8b181d6-3b16-4a41-99fa-0713b94a34d9'
                    ) AS inner
                    ON inner.category_id = category.id
            ",
        )
        .bind(self.id.to_string())
        .fetch(pool)
        .map_ok(|row| -> Result<(), Error> {
            let mut category = TripCategory {
                category: Category {
                    id: Uuid::try_parse(row.try_get("category_id")?)?,
                    name: row.try_get("category_name")?,
                    description: row.try_get("category_description")?,
                    items: None,
                },
                items: None,
            };

            match row.try_get("item_id")? {
                None => {
                    // we have an empty (unused) category which has NULL values
                    // for the item_id column
                    category.items = Some(vec![]);
                    categories.push(category);
                }
                Some(item_id) => {
                    let item = TripItem {
                        item: Item {
                            id: Uuid::try_parse(item_id)?,
                            name: row.try_get("item_name")?,
                            description: row.try_get("item_description")?,
                            weight: row.try_get("item_weight")?,
                            category_id: category.category.id,
                        },
                        picked: row.try_get("item_is_picked")?,
                        packed: row.try_get("item_is_packed")?,
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

impl TryFrom<SqliteRow> for TripType {
    type Error = Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let id: Uuid = Uuid::try_parse(row.try_get("id")?)?;
        let name: String = row.try_get::<&str, _>("name")?.to_string();
        let active: bool = row.try_get("active")?;

        Ok(Self { id, name, active })
    }
}

#[derive(Debug)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    items: Option<Vec<Item>>,
}

impl TryFrom<SqliteRow> for Category {
    type Error = Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let name: &str = row.try_get("name")?;
        let description: &str = row.try_get("description")?;
        let id: Uuid = Uuid::try_parse(row.try_get("id")?)?;

        Ok(Category {
            id,
            name: name.to_string(),
            description: description.to_string(),
            items: None,
        })
    }
}

impl<'a> Category {
    pub fn items(&'a self) -> &'a Vec<Item> {
        self.items
            .as_ref()
            .expect("you need to call populate_items()")
    }

    pub fn total_weight(&self) -> u32 {
        self.items().iter().map(|item| item.weight).sum()
    }

    pub async fn populate_items(
        &'a mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
        let items = sqlx::query(&format!(
            "SELECT
                id,name,weight,description,category_id
            FROM inventory_items
            WHERE category_id = '{id}'",
            id = self.id
        ))
        .fetch(pool)
        .map_ok(std::convert::TryInto::try_into)
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
    pub description: String,
    pub weight: u32,
    pub category_id: Uuid,
}

impl TryFrom<SqliteRow> for Item {
    type Error = Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let name: &str = row.try_get("name")?;
        let description: &str = row.try_get("description")?;
        let weight: u32 = row.try_get("weight")?;
        let id: Uuid = Uuid::try_parse(row.try_get("id")?)?;
        let category_id: Uuid = Uuid::try_parse(row.try_get("category_id")?)?;

        Ok(Item {
            id,
            name: name.to_string(),
            weight,
            description: description.to_string(),
            category_id,
        })
    }
}

impl Item {
    pub async fn find(pool: &sqlx::Pool<sqlx::Sqlite>, id: Uuid) -> Result<Option<Item>, Error> {
        let item: Result<Result<Item, Error>, sqlx::Error> = sqlx::query(
            "SELECT * FROM inventory_items AS item
            WHERE item.id = ?",
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .map_ok(std::convert::TryInto::try_into)
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
        weight: u32,
    ) -> Result<Option<Uuid>, Error> {
        let id: Result<Result<Uuid, Error>, sqlx::Error> = sqlx::query(
            "UPDATE inventory_items AS item
            SET
                name = ?,
                weight = ?
            WHERE item.id = ?
            RETURNING inventory_items.category_id AS id
            ",
        )
        .bind(name)
        .bind(weight)
        .bind(id.to_string())
        .fetch_one(pool)
        .map_ok(|row| {
            let id: &str = row.try_get("id")?;
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
