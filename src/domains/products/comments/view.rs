use maud::{html, Markup};
use uuid::Uuid;

use crate::components::{
    types::{Name, Url},
    Render as _, Text, TextListWithDate,
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
        let comments = TextListWithDate::builder()
            .ident("comments")
            .name(Name {
                singular: "comment".into(),
                plural: "comments".into(),
            })
            .rows(
                product
                    .comments
                    .iter()
                    .map(|comment| {
                        (
                            comment.id.clone(),
                            comment.date.clone(),
                            comment.content.clone(),
                            |comment_id| {
                                Url(format!(
                                    "/products/{product_id}/comments/{comment_id}/delete",
                                    product_id = product.id
                                ))
                            },
                            |comment_id| {
                                Url(format!(
                                    "/products/{product_id}/comments/{comment_id}/edit",
                                    product_id = product.id
                                ))
                            },
                        )
                            .into()
                    })
                    .collect(),
            )
            .new_row(Url(format!("/products/{id}/comments/new", id = product.id)))
            .build();

        html!((comments.render()))
    }
}

pub(crate) struct EditComment;

impl EditComment {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_product_comment",
        fields(component = "Product")
    )]
    pub fn build(product_id: Uuid, comment: &super::model::Comment) -> Markup {
        let text = Text::builder()
            .id(comment.id)
            .initial_content(comment.content.clone())
            .save(|comment_id| {
                Url(format!(
                    "/products/{product_id}/comments/{comment_id}/edit/save",
                    product_id = product_id
                ))
            })
            .cancel(|_comment_id| Url(format!("/products/{product_id}")))
            .build();

        html!(
            div
                ."m-2"
            {
                (text.render())
            }
        )
    }
}
