use crate::error::Error;
use crate::{db, Context};

use rust_decimal::Decimal;
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

pub(crate) struct DbLink {
    pub id: Uuid,
    pub name: String,
    pub url: String,
}

impl TryFrom<DbLink> for Link {
    type Error = Error;

    fn try_from(row: DbLink) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            url: row.url,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Link {
    #[allow(dead_code)]
    pub id: Uuid,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub(crate) struct Product {
    #[allow(dead_code)]
    pub id: Uuid,
    pub name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    pub links: Vec<Link>,
    pub price: Option<crate::models::Currency>,
    pub purchase_date: Option<time::Date>,
    pub purchase_from: Option<String>,
    pub comments: Vec<Comment>,
}

impl Product {
    #[tracing::instrument]
    pub async fn find(ctx: &Context, pool: &db::Pool, id: Uuid) -> Result<Option<Self>, Error> {
        if cfg!(false) {
            pub(crate) struct Row {
                pub id: Uuid,
                pub name: String,
                pub description: Option<String>,
                pub price: Option<sqlx::postgres::types::PgMoney>,
                pub purchase_from: Option<String>,
                pub purchase_date: Option<time::Date>,
                pub link_id: Option<Uuid>,
                pub link_name: Option<String>,
                pub link_url: Option<String>,
                pub comment_id: Option<Uuid>,
                pub comment_content: Option<String>,
                pub comment_date: Option<time::Date>,
            }

            struct RowParsed {
                pub id: Uuid,
                pub name: String,
                pub description: Option<String>,
                pub link: Option<Link>,
                pub price: Option<crate::models::Currency>,
                pub purchase_from: Option<String>,
                pub purchase_date: Option<time::Date>,
                pub comment: Option<Comment>,
            }

            impl TryFrom<Row> for RowParsed {
                type Error = Error;

                fn try_from(row: Row) -> Result<Self, Self::Error> {
                    Ok(Self {
                        id: row.id,
                        name: row.name,
                        description: row.description,
                        link: row.link_id.map(|id| Link {
                            id,
                            name: row.link_name.unwrap(),
                            url: row.link_url.unwrap(),
                        }),
                        price: row.price.map(|price| {
                            let cents = price.0;
                            crate::models::Currency::Eur(
                                Decimal::try_from_i128_with_scale(cents.into(), 2).unwrap(),
                            )
                        }),
                        purchase_from: row.purchase_from,
                        purchase_date: row.purchase_date,
                        comment: row.comment_id.map(|id| Comment {
                            id,
                            content: row.comment_content.unwrap(),
                            date: row.comment_date.unwrap(),
                        }),
                    })
                }
            }

            let mut results = crate::query_all!(
                &db::QueryClassification {
                    query_type: db::QueryType::Select,
                    component: db::Component::Inventory,
                },
                pool,
                Row,
                RowParsed,
                r#"
                SELECT
                    product.id AS id,
                    product.name AS name,
                    product.description AS description,
                    product.price AS price,
                    product.bought_at AS purchase_date,
                    product.bought_from AS purchase_from,
                    link.id AS "link_id?",
                    link.name AS "link_name?",
                    link.url AS "link_url?",
                    comment.id AS "comment_id?",
                    comment.content AS "comment_content?",
                    comment.date AS "comment_date?"
                FROM products AS product
                LEFT JOIN product_links AS link
                    ON link.product_id = product.id
                LEFT JOIN product_comments AS comment
                    ON comment.product_id = product.id
                WHERE
                    product.id = $1
                    AND product.user_id = $2"#,
                id,
                ctx.user.id,
            )
            .await?;

            let mut product = match results.pop() {
                None => return Ok(None),
                Some(product) => Product {
                    id: product.id,
                    name: product.name,
                    description: product.description,
                    links: product
                        .link
                        .map(|link| vec![link])
                        .unwrap_or_else(|| vec![]),
                    price: product.price,
                    purchase_date: product.purchase_date,
                    purchase_from: product.purchase_from,
                    comments: product
                        .comment
                        .map(|comment| vec![comment])
                        .unwrap_or_else(|| vec![]),
                },
            };

            let mut seen_link_ids = product
                .links
                .get(0)
                .map(|link| vec![link.id])
                .unwrap_or_else(|| vec![]);

            for result in results.iter_mut() {
                if let Some(link) = result.link.take() {
                    if !seen_link_ids.contains(&link.id) {
                        seen_link_ids.push(link.id);
                        product.links.push(link);
                    }
                }
            }

            let mut seen_comment_ids = product
                .comments
                .get(0)
                .map(|comment| vec![comment.id])
                .unwrap_or_else(|| vec![]);

            for result in results.iter_mut() {
                if let Some(comment) = result.comment.take() {
                    if !seen_comment_ids.contains(&comment.id) {
                        seen_comment_ids.push(comment.id);
                        product.comments.push(comment);
                    }
                }
            }

            Ok(Some(product))
        } else {
            #[derive(Debug)]
            pub(crate) struct Row {
                pub id: Uuid,
                pub name: String,
                pub description: Option<String>,
                pub price: Option<sqlx::postgres::types::PgMoney>,
                pub purchase_from: Option<String>,
                pub purchase_date: Option<time::Date>,
                pub link_ids: Vec<Uuid>,
                pub link_names: Vec<String>,
                pub link_urls: Vec<String>,
                pub comment_ids: Vec<Uuid>,
                pub comment_contents: Vec<String>,
                pub comment_dates: Vec<time::Date>,
            }

            impl TryFrom<Row> for Product {
                type Error = Error;

                fn try_from(row: Row) -> Result<Self, Self::Error> {
                    Ok(Product {
                        id: row.id,
                        name: row.name,
                        description: row.description,
                        price: row.price.map(|price| {
                            let cents = price.0;
                            crate::models::Currency::Eur(
                                Decimal::try_from_i128_with_scale(cents.into(), 2).unwrap(),
                            )
                        }),
                        purchase_date: row.purchase_date,
                        purchase_from: row.purchase_from,
                        links: row
                            .link_ids
                            .into_iter()
                            .zip(row.link_names)
                            .zip(row.link_urls)
                            .map(|((id, name), url)| Link { id, name, url })
                            .collect(),
                        comments: row
                            .comment_ids
                            .into_iter()
                            .zip(row.comment_contents)
                            .zip(row.comment_dates)
                            .map(|((id, content), date)| Comment { id, content, date })
                            .collect(),
                    })
                }
            }

            let result = crate::query_one_file!(
                &db::QueryClassification {
                    query_type: db::QueryType::Select,
                    component: db::Component::Inventory,
                },
                pool,
                Row,
                Product,
                "./sql/single_product.sql",
                id,
                ctx.user.id,
            )
            .await?;

            Ok(result)
        }
    }
}
