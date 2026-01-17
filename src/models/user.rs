use crate::Error;
use uuid::Uuid;

use crate::db;

#[derive(Debug, Clone)]
pub(crate) struct User {
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
pub(crate) struct DbUserRow {
    id: Uuid,
    username: String,
    fullname: String,
}

impl TryFrom<DbUserRow> for User {
    type Error = Error;

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
    pub async fn find_by_name(pool: &db::Pool, name: &str) -> Result<Option<Self>, Error> {
        crate::query_one!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::User,
            },
            pool,
            DbUserRow,
            Self,
            "SELECT id,username,fullname FROM users WHERE username = $1",
            name
        )
        .await
    }
}

#[tracing::instrument]
pub async fn create(pool: &db::Pool, user: NewUser<'_>) -> Result<Uuid, Error> {
    let id = Uuid::new_v4();

    crate::execute!(
        &db::QueryClassification {
            query_type: db::QueryType::Insert,
            component: db::Component::User,
        },
        pool,
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
