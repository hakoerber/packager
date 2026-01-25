use uuid::Uuid;

use crate::{DatabaseError, RunError};

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
    id: Uuid,
    username: String,
    fullname: String,
}

impl TryFrom<DbUserRow> for User {
    type Error = RunError;

    fn try_from(row: DbUserRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            username: row.username,
            fullname: row.fullname,
        })
    }
}

impl User {
    #[tracing::instrument]
    pub async fn find_by_name(pool: &database::Pool, name: &str) -> Result<Option<Self>, RunError> {
        database::query_one!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: database::Component::User,
            },
            pool,
            DbUserRow,
            Self,
            RunError,
            "SELECT id,username,fullname FROM users WHERE username = $1",
            name
        )
        .await
    }
}

#[tracing::instrument]
pub async fn create(pool: &database::Pool, user: NewUser<'_>) -> Result<Uuid, DatabaseError> {
    let id = Uuid::new_v4();

    database::execute!(
        &database::QueryClassification {
            query_type: database::QueryType::Insert,
            component: database::Component::User,
        },
        pool,
        DatabaseError,
        "INSERT INTO users
            (id, username, fullname)
        VALUES
            ($1, $2, $3)",
        id,
        user.username,
        user.fullname
    )
    .await?;

    Ok(id)
}
