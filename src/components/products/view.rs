use maud::{html, Markup};

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
        html!(
            div ."p-8" {
                table
                    ."table"
                    ."table-auto"
                    ."border-collapse"
                    ."border-spacing-0"
                    ."border"
                    ."w-full"
                {
                    tbody {
                        tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                            td ."border" ."p-2" { "Name" }
                            td ."border" ."p-2" { (product.name) }
                        }
                        tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                            td ."border" ."p-2" { "Description" }
                            td ."border" ."p-2" { (product.description.clone().unwrap_or(String::new())) }
                        }
                    }
                }
            }

            div ."p-8" {
                table
                    ."table"
                    ."table-auto"
                    ."border-collapse"
                    ."border-spacing-0"
                    ."border"
                    ."w-full"
                {
                    tbody {
                        @for link in &product.links {
                            tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                                td ."border" ."p-2" { (link.name) }
                                td ."border" ."p-2" { a href=(link.url) { (link.url) } }
                            }
                        }
                    }
                }
            }
        )
    }
}
