use crate::{Context, RequestError, RunError};

use serde::Deserialize;
use uuid::Uuid;

pub struct DbComment {
    pub id: Uuid,
    pub content: String,
    pub date: time::Date,
}

impl TryFrom<DbComment> for Comment {
    type Error = RunError;

    fn try_from(row: DbComment) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            content: row.content,
            date: row.date,
        })
    }
}

#[derive(Debug)]
pub struct Comment {
    #[allow(dead_code)]
    pub id: Uuid,
    pub content: String,
    pub date: time::Date,
}

impl Comment {
    #[tracing::instrument]
    pub async fn create(
        ctx: &Context,
        pool: &database::Pool,
        product_id: Uuid,
        new_comment: NewComment,
    ) -> Result<Uuid, RunError> {
        database::execute_returning_uuid!(
            &database::QueryClassification {
                query_type: database::QueryType::Insert,
                component: crate::Component::Inventory,
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
        pool: &database::Pool,
        product_id: Uuid,
        comment_id: Uuid,
    ) -> Result<bool, RunError> {
        let results = database::execute!(
            &database::QueryClassification {
                query_type: database::QueryType::Delete,
                component: crate::Component::Trips,
            },
            pool,
            RunError,
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
        pool: &database::Pool,
        product_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Option<Self>, RunError> {
        database::query_one!(
            &database::QueryClassification {
                query_type: database::QueryType::Select,
                component: crate::Component::Todo,
            },
            pool,
            DbComment,
            Self,
            RunError,
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
        pool: &database::Pool,
        product_id: Uuid,
        comment_id: Uuid,
        update_comment: UpdateComment,
    ) -> Result<(), RunError> {
        let result: Result<_, RunError> = database::execute_returning_optional_uuid!(
            &database::QueryClassification {
                query_type: database::QueryType::Update,
                component: crate::Component::Inventory,
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
                RunError::Request(RequestError::NotFound {
                    message: format!("comment with id {comment_id} not found"),
                })
            })
        })??;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct NewComment {
    #[serde(rename = "new-comment-content")]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateComment {
    #[serde(rename = "new-content")]
    pub content: String,
}
