use super::Error;
use uuid::Uuid;

use crate::sqlite;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub fullname: String,
}

#[derive(Debug)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub fullname: &'a str,
}

#[derive(Debug)]
pub struct DbUserRow {
    id: String,
    username: String,
    fullname: String,
}

impl TryFrom<DbUserRow> for User {
    type Error = Error;

    fn try_from(row: DbUserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: Uuid::try_parse(&row.id)?,
            username: row.username,
            fullname: row.fullname,
        })
    }
}

impl User {
    #[tracing::instrument]
    pub async fn find_by_name(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
    ) -> Result<Option<Self>, Error> {
        crate::query_one!(
            &sqlite::QueryClassification {
                query_type: sqlite::QueryType::Select,
                component: sqlite::Component::User,
            },
            pool,
            DbUserRow,
            Self,
            "SELECT id,username,fullname FROM users WHERE username = ?",
            name
        )
        .await
    }
}

#[tracing::instrument]
pub async fn create(pool: &sqlite::Pool, user: NewUser<'_>) -> Result<Uuid, Error> {
    let id = Uuid::new_v4();
    let id_param = id.to_string();

    crate::execute!(
        &sqlite::QueryClassification {
            query_type: sqlite::QueryType::Insert,
            component: sqlite::Component::User,
        },
        pool,
        "INSERT INTO users
            (id, username, fullname)
        VALUES
            (?, ?, ?)",
        id_param,
        user.username,
        user.fullname
    )
    .await?;

    Ok(id)
}
