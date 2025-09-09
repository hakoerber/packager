use maud::{html, Markup};

use crate::components::{
    listtable::{Currency, Date, Link, Raw, Render as _, Url},
    ListTable,
};

use super::model;

pub(crate) struct Product;

impl Product {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_product",
        fields(component = "Product")
        skip(product)
    )]
    pub fn build(product: &model::Product) -> Markup {
        let info = ListTable::from_rows(vec![
            Box::new((Raw("Name".to_owned()), Raw(product.name.to_owned()))),
            Box::new((
                Raw("Description".to_owned()),
                product.description.as_ref().map(|p| Raw(p.to_owned())),
            )),
            Box::new((
                Raw("Price".to_owned()),
                product.price.as_ref().map(|p| Currency(p.clone())),
            )),
            Box::new((
                Raw("Purchased at".to_owned()),
                product.purchase_date.as_ref().map(|p| Date(p.to_owned())),
            )),
            Box::new((
                Raw("Purchased from".to_owned()),
                product.purchase_from.as_ref().map(|p| Raw(p.to_owned())),
            )),
        ]);

        let links = ListTable::from_rows(
            product
                .links
                .iter()
                .map(|link| {
                    Box::new((
                        Raw(link.name.clone()),
                        Link {
                            name: None,
                            url: Url(link.url.clone()),
                        },
                    )) as _
                })
                .collect(),
        );

        let comments = ListTable::from_rows(
            product
                .comments
                .iter()
                .map(|comment| {
                    Box::new((Date(comment.date.clone()), Raw(comment.content.clone()))) as _
                })
                .collect(),
        );

        html!(
            div ."p-8" {
                (info.render())
            }

            div ."p-8" {
                (links.render())
            }

            div ."p-8" {
                (comments.render())
            }
        )
    }
}
