use crate::{db, error::Error, Context};

use serde::Deserialize;
use uuid::Uuid;

pub(crate) struct DbComment {
    pub id: Uuid,
    pub content: String,
    pub date: time::Date,
}

impl TryFrom<DbComment> for Comment {
    type Error = Error;

    fn try_from(row: DbComment) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            content: row.content,
            date: row.date,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Comment {
    #[allow(dead_code)]
    pub id: Uuid,
    pub content: String,
    pub date: time::Date,
}

impl Comment {
    #[tracing::instrument]
    pub async fn new(
        ctx: &Context,
        pool: &db::Pool,
        product_id: Uuid,
        new_comment: NewComment,
    ) -> Result<Uuid, Error> {
        crate::execute_returning_uuid!(
            &db::QueryClassification {
                query_type: db::QueryType::Insert,
                component: db::Component::Inventory,
            },
            pool,
            "INSERT INTO product_comments
                (id, content, date, product_id)
            VALUES
                (uuidv4(), $1, current_date, $2)
            RETURNING id",
            new_comment.content,
            product_id,
        )
        .await
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewComment {
    #[serde(rename = "new-comment-content")]
    pub content: String,
}
