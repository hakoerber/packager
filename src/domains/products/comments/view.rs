use maud::{html, Markup};

use crate::components::{
    types::{Date, Name, Raw, Url},
    List, Render as _,
};

pub(crate) struct Comments;

impl Comments {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_product_comments",
        fields(component = "Product")
        skip(product)
    )]
    pub fn build(product: &super::super::model::Product) -> Markup {
        let comments = List::builder()
            .ident("comments")
            .name(Name {
                singular: "comment".into(),
                plural: "comments".into(),
            })
            .rows(
                product
                    .comments
                    .iter()
                    .map(|comment| (Date(comment.date.clone()), Raw(comment.content.clone())))
                    .collect(),
            )
            .new_row(Url(format!("/products/{id}/comments/new", id = product.id)))
            .build();

        html!((comments.render()))
    }
}
