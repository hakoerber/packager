use super::Error;
use uuid::Uuid;

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
    //#[tracing::instrument]
    pub async fn find_by_name(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        name: &str,
    ) -> Result<Option<Self>, Error> {
        sqlx::query_as!(
            DbUserRow,
            "SELECT id,username,fullname FROM users WHERE username = ?",
            name
        )
        .fetch_optional(pool)
        .await?
        .map(|row: DbUserRow| row.try_into())
        .transpose()
    }
}

//#[tracing::instrument]
pub async fn create(pool: &sqlx::Pool<sqlx::Sqlite>, user: NewUser<'_>) -> Result<Uuid, Error> {
    let id = Uuid::new_v4();
    let id_param = id.to_string();

    sqlx::query!(
        "INSERT INTO users
            (id, username, fullname)
        VALUES
            (?, ?, ?)",
        id_param,
        user.username,
        user.fullname
    )
    .execute(pool)
    .await?;

    Ok(id)
}
