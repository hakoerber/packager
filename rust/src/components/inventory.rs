use maud::{html, Markup};

use crate::models::*;
use crate::State;
use uuid::uuid;

pub struct Inventory {
    doc: Markup,
}

impl Inventory {
    pub async fn build(state: State, categories: Vec<Category>) -> Result<Self, Error> {
        let doc = html!(
            div id="pkglist-item-manager" {
                div ."p-8" ."grid" ."grid-cols-4" ."gap-3" {
                    div ."col-span-2" {
                        ({<InventoryCategoryList as Into<Markup>>::into(InventoryCategoryList::build(&categories).await?)})
                    }
                    div ."col-span-2" {
                        h1 ."text-2xl" ."mb-5" ."text-center" { "Items" }
                        @if state.active_category_id.is_some() {
                            ({<InventoryItemList as Into<Markup>>::into(InventoryItemList::build(categories.iter().find(|category| category.active).unwrap().items()).await?)})
                        }
                        ({<InventoryNewItemForm as Into<Markup>>::into(InventoryNewItemForm::build(&state, &categories).await?)})

                    }
                }
            }
        );

        Ok(Self { doc })
    }
}

impl Into<Markup> for Inventory {
    fn into(self) -> Markup {
        self.doc
    }
}

pub struct InventoryCategoryList {
    doc: Markup,
}

impl InventoryCategoryList {
    pub async fn build(categories: &Vec<Category>) -> Result<Self, Error> {
        let biggest_category_weight: u32 = categories
            .iter()
            .map(|category| category.total_weight())
            .max()
            .unwrap_or(1);

        let doc = html!(
            div {
                h1 ."text-2xl" ."mb-5" ."text-center" { "Categories" }
                table
                    ."table"
                    ."table-auto"
                    ."border-collapse"
                    ."border-spacing-0"
                    ."border"
                    ."w-full"
                {

                    colgroup {
                        col style="width:50%" {}
                        col style="width:50%" {}
                    }
                    thead ."bg-gray-200" {
                        tr ."h-10" {
                            th ."border" ."p-2" { "Name" }
                            th ."border" ."p-2" { "Weight" }
                        }
                    }
                    tbody {
                        @for category in categories {
                            tr class={@if category.active {
                                    "h-10 hover:bg-purple-100 m-3 h-full outline outline-2 outline-indigo-300"
                                } @else {
                                    "h-10 hover:bg-purple-100 m-3 h-full"
                                }} {

                                td
                                    class=@if category.active {
                                       "border p-0 m-0 font-bold"
                                    } @else {
                                       "border p-0 m-0"
                                    } {
                                    a
                                        id="select-category"
                                        href=(
                                            format!(
                                                "/inventory/category/{id}",
                                                id=category.id
                                            )
                                        )
                                        // hx-post=(
                                        //     format!(
                                        //         "/inventory/category/{id}/items",
                                        //         id=category.id
                                        //     )
                                        // )
                                        // hx-swap="outerHTML"
                                        // hx-target="#items"
                                        ."inline-block" ."p-2" ."m-0" ."w-full"
                                        {
                                            (category.name.clone())
                                        }
                                }
                                td ."border" ."p-2" ."m-0" style="position:relative;" {
                                    p {
                                        (category.total_weight().to_string())
                                    }
                                    div ."bg-blue-600" ."h-1.5"
                                        style=(
                                            format!(
                                                "width: {width}%;position:absolute;left:0;bottom:0;right:0;",
                                                width=(
                                                    category.total_weight() as f32
                                                    / biggest_category_weight as f32
                                                    * 100.0
                                                )
                                            )
                                        ) {}
                                }
                            }
                        }
                        tr ."h-10" ."hover:bg-purple-200" ."bg-gray-300" ."font-bold" {
                            td ."border" ."p-0" ."m-0" {
                                p ."p-2" ."m-2" { "Sum" }
                            }
                            td ."border" ."p-0" ."m-0" {
                                p ."p-2" ."m-2" {
                                    (categories.iter().map(|category| category.total_weight()).sum::<u32>().to_string())
                                }
                            }
                        }
                    }
                }
            }
        );

        Ok(Self { doc })
    }
}

impl Into<Markup> for InventoryCategoryList {
    fn into(self) -> Markup {
        self.doc
    }
}

pub struct InventoryItemList {
    doc: Markup,
}

impl InventoryItemList {
    pub async fn build(items: &Vec<Item>) -> Result<Self, Error> {
        let biggest_item_weight: u32 = items.iter().map(|item| item.weight).max().unwrap_or(1);
        let doc = html!(
            div #items {
                @if items.is_empty() {
                    p ."text-lg" ."text-center" ."py-5" ."text-gray-400" { "[Empty]" }
                } @else {
                    table
                        ."table"
                        ."table-auto"
                        ."border-collapse"
                        ."border-spacing-0"
                        ."border"
                        ."w-full"
                    {
                        thead ."bg-gray-200" {
                            tr ."h-10" {
                                th ."border" ."p-2" { "Name" }
                                th ."border" ."p-2" { "Weight" }
                            }
                        }
                        tbody {
                            @for item in items {
                                tr ."h-10" ."even:bg-gray-100" ."hover:bg-purple-100" {
                                    td ."border" ."p-0" {
                                        a
                                            ."p-2" ."w-full" ."inline-block"
                                            href=(
                                                format!("/inventory/item/{id}/", id=item.id)
                                            ) {

                                                (item.name.clone())
                                            }
                                    }
                                    td ."border" ."p-2" style="position:relative;" {
                                        p { (item.weight.to_string()) }
                                        div ."bg-blue-600" ."h-1.5" style=(format!("
                                        width: {width}%;
                                        position:absolute;
                                        left:0;
                                        bottom:0;
                                        right:0;", width=(item.weight as f32 / biggest_item_weight as f32 * 100.0))) {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        );

        Ok(Self { doc })
    }
}

impl Into<Markup> for InventoryItemList {
    fn into(self) -> Markup {
        self.doc
    }
}

pub struct InventoryNewItemForm {
    doc: Markup,
}

impl InventoryNewItemForm {
    pub async fn build(state: &State, categories: &Vec<Category>) -> Result<Self, Error> {
        let doc = html!(

            form
                name="new-item"
                id="new-item"
                action="/inventory/item/"
                target="_self"
                method="post"
                ."mt-8" ."p-5" ."border-2" ."border-gray-200" {
                div ."mb-5" ."flex" ."flex-row" ."items-center" {
                    span ."mdi" ."mdi-playlist-plus" ."text-2xl" ."mr-4" {}
                    p ."inline" ."text-xl" { "Add new item" }
                }
                div ."w-11/12" ."mx-auto" {
                    div ."pb-8" {
                        div ."flex" ."flex-row" ."justify-center" ."items-start"{
                            label for="item-name" .font-bold ."w-1/2" ."p-2" ."text-center" { "Name" }
                            span ."w-1/2" {
                                input type="text" id="item-name" name="name"
                                    ."block"
                                    ."w-full"
                                    ."p-2"
                                    ."bg-gray-50"
                                    ."border-2"
                                    ."rounded"
                                    ."focus:outline-none"
                                    ."focus:bg-white"
                                    ."focus:border-purple-500"
                                    {
                                    }
                            }
                        }
                    }
                    div ."flex" ."flex-row" ."justify-center" ."items-center" ."pb-8" {
                        label for="item-weight" .font-bold ."w-1/2" .text-center { "Weight" }
                        span ."w-1/2" {
                            input
                                type="text"
                                id="item-weight"
                                name="weight"
                                ."block"
                                ."w-full"
                                ."p-2"
                                ."bg-gray-50"
                                ."border-2"
                                ."border-gray-300"
                                ."rounded"
                                ."focus:outline-none"
                                ."focus:bg-white"
                                ."focus:border-purple-500"
                                {
                                }
                        }
                    }
                    div ."flex" ."flex-row" ."justify-center" ."items-center" ."pb-8" {
                        label for="item-category" .font-bold ."w-1/2" .text-center { "Category" }
                        span ."w-1/2" {
                            select
                                    id="item-category"
                                    name="category"
                                    ."block"
                                    ."w-full"
                                    ."p-2"
                                    ."bg-gray-50"
                                    ."border-2"
                                    ."border-gray-300"
                                    ."rounded"
                                    ."focus:outline-none"
                                    ."focus:bg-white"
                                    ."focus:border-purple-500" {
                                @for category in categories {
                                    @if state.active_category_id.map_or(false, |id| id == category.id) {

                                        option value=(category.id) selected="true" {
                                            (category.name)
                                        }
                                    } @else {
                                        option value=(category.id) {
                                            (category.name)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    input type="submit" value="Add"
                        ."py-2"
                        ."border-2"
                        ."rounded"
                        ."border-gray-300"
                        ."mx-auto"
                        ."w-full" {
                    }
                }
            }
        );

        Ok(Self { doc })
    }
}

impl Into<Markup> for InventoryNewItemForm {
    fn into(self) -> Markup {
        self.doc
    }
}
// impl InventoryItemList {
//     pub fn to_string(self) -> String {
//         self.doc.into_string()
//     }
// }
//ItemList
