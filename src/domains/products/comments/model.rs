use crate::{Context, RequestError, db, error::Error};

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
    pub async fn create(
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

    #[tracing::instrument]
    pub async fn delete(
        ctx: &Context,
        pool: &db::Pool,
        product_id: Uuid,
        comment_id: Uuid,
    ) -> Result<bool, Error> {
        let results = crate::execute!(
            &db::QueryClassification {
                query_type: db::QueryType::Delete,
                component: db::Component::Trips,
            },
            pool,
            "DELETE FROM product_comments AS comment
            WHERE comment.product_id = $1
                AND comment.id = $2
            AND EXISTS(SELECT * FROM products WHERE id = $1 AND user_id = $3)
            ",
            product_id,
            comment_id,
            ctx.user.id,
        )
        .await?;

        Ok(results.rows_affected() != 0)
    }

    #[tracing::instrument]
    pub async fn find(
        ctx: &Context,
        pool: &db::Pool,
        product_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Option<Self>, Error> {
        crate::query_one!(
            &db::QueryClassification {
                query_type: db::QueryType::Select,
                component: db::Component::Todo,
            },
            pool,
            DbComment,
            Self,
            r"
                SELECT
                    comment.id AS id,
                    comment.content AS content,
                    comment.date AS date
                FROM product_comments AS comment
                INNER JOIN products AS product
                    ON product.id = comment.product_id
                WHERE
                    comment.id = $1
                    AND product.id = $2
                    AND product.user_id = $3
            ",
            comment_id,
            product_id,
            ctx.user.id,
        )
        .await
    }

    #[tracing::instrument]
    pub async fn update(
        ctx: &Context,
        pool: &db::Pool,
        product_id: Uuid,
        comment_id: Uuid,
        update_comment: UpdateComment,
    ) -> Result<(), Error> {
        let result: Result<_, Error> = crate::execute_returning_optional_uuid!(
            &db::QueryClassification {
                query_type: db::QueryType::Update,
                component: db::Component::Inventory,
            },
            pool,
            r"
                UPDATE product_comments AS comment
                    SET content = $1
                WHERE comment.product_id = $2
                AND comment.id = $3
                AND EXISTS(SELECT 1 FROM products WHERE id = $2 AND user_id = $4)
                RETURNING comment.id AS id
            ",
            update_comment.content,
            product_id,
            comment_id,
            ctx.user.id
        )
        .await;

        let _id = result.map(|e| {
            e.ok_or_else(|| {
                Error::Request(RequestError::NotFound {
                    message: format!("comment with id {comment_id} not found"),
                })
            })
        })??;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewComment {
    #[serde(rename = "new-comment-content")]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateComment {
    #[serde(rename = "new-content")]
    pub content: String,
}
