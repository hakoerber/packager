use crate::error::Error;
use crate::{db, Context};

use uuid::Uuid;

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

#[derive(Debug)]
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
}

impl Product {
    #[tracing::instrument]
    pub async fn find(ctx: &Context, pool: &db::Pool, id: Uuid) -> Result<Option<Self>, Error> {
        pub(crate) struct Row {
            pub id: Uuid,
            pub name: String,
            pub description: Option<String>,
            pub link_id: Option<Uuid>,
            pub link_name: Option<String>,
            pub link_url: Option<String>,
        }

        struct RowParsed {
            pub id: Uuid,
            pub name: String,
            pub description: Option<String>,
            pub link: Option<Link>,
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
                    link.id AS "link_id?",
                    link.name AS "link_name?",
                    link.url AS "link_url?"
                FROM products AS product
                LEFT JOIN product_links AS link
                    ON link.product_id = product.id
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
            },
        };

        for result in results {
            product.links.push(result.link.unwrap())
        }

        println!("{product:?}");

        Ok(Some(product))
    }
}
