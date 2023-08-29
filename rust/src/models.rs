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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TripAttribute {
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
        })
    }
}

impl<'a> Trip {
    pub fn types(&'a self) -> &Vec<TripType> {
        self.types
            .as_ref()
            .expect("you need to call load_triptypes()")
    }
}

impl<'a> Trip {
    pub async fn load_triptypes(
        &'a mut self,
        pool: &sqlx::Pool<sqlx::Sqlite>,
    ) -> Result<(), Error> {
        let types = sqlx::query(
            "
            SELECT
                type.id as id,
                type.name as name,
                CASE WHEN inner.id IS NOT NULL THEN true ELSE false END AS active
            FROM triptypes AS type
                LEFT JOIN (
                    SELECT type.id as id, type.name as name
                    FROM trips as trip
                    INNER JOIN trips_to_triptypes as ttt
                        ON ttt.trip_id = trip.id
                    INNER JOIN triptypes AS type
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
            FROM inventoryitems
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
            "SELECT * FROM inventoryitems AS item
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
            "UPDATE inventoryitems AS item
            SET
                name = ?,
                weight = ?
            WHERE item.id = ?
            RETURNING inventoryitems.category_id AS id
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
