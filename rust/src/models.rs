use sqlx::{sqlite::SqliteRow, Row};
use std::convert;
use std::error;
use std::fmt;
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

pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub start_date: time::Date,
    pub end_date: time::Date,
}

impl TryFrom<SqliteRow> for Trip {
    type Error = Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        let name: &str = row.try_get("name")?;
        let id: &str = row.try_get("id")?;
        let start_date: time::Date = row.try_get("start_date")?;
        let end_date: time::Date = row.try_get("start_date")?;
        let id: Uuid = Uuid::try_parse(id)?;

        Ok(Trip {
            name: name.to_string(),
            id,
            start_date,
            end_date,
        })
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
