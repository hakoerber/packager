use maud::{html, Markup};

use crate::models::*;
use crate::ClientState;
use uuid::{uuid, Uuid};

pub struct Inventory {
    doc: Markup,
}

impl Inventory {
    pub async fn build(state: ClientState, categories: Vec<Category>) -> Result<Self, Error> {
        let doc = html!(
            div id="pkglist-item-manager" {
                div ."p-8" ."grid" ."grid-cols-4" ."gap-3" {
                    div ."col-span-2" {
                        ({<InventoryCategoryList as Into<Markup>>::into(InventoryCategoryList::build(&categories))})
                    }
                    div ."col-span-2" {
                        h1 ."text-2xl" ."mb-5" ."text-center" { "Items" }
                        @if state.active_category_id.is_some() {
                            ({<InventoryItemList as Into<Markup>>::into(InventoryItemList::build(categories.iter().find(|category| category.active).unwrap().items(), state.edit_item))})
                        }
                        ({<InventoryNewItemForm as Into<Markup>>::into(InventoryNewItemForm::build(&state, &categories))})

                    }
                }
            }
        );

        Ok(Self { doc })
    }
}

impl From<Inventory> for Markup {
    fn from(val: Inventory) -> Self {
        val.doc
    }
}

pub struct InventoryCategoryList {
    doc: Markup,
}

impl InventoryCategoryList {
    pub fn build(categories: &Vec<Category>) -> Self {
        let biggest_category_weight: u32 = categories
            .iter()
            .map(Category::total_weight)
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
                                                    f64::from(category.total_weight())
                                                    / f64::from(biggest_category_weight)
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
                                    (categories.iter().map(Category::total_weight).sum::<u32>().to_string())
                                }
                            }
                        }
                    }
                }
            }
        );

        Self { doc }
    }
}

impl From<InventoryCategoryList> for Markup {
    fn from(val: InventoryCategoryList) -> Self {
        val.doc
    }
}

pub struct InventoryItemList {
    doc: Markup,
}

impl InventoryItemList {
    pub fn build(items: &Vec<Item>, edit_item: Option<Uuid>) -> Self {
        let biggest_item_weight: u32 = items.iter().map(|item| item.weight).max().unwrap_or(1);
        let doc = html!(
            div #items {
                @if items.is_empty() {
                    p ."text-lg" ."text-center" ."py-5" ."text-gray-400" { "[Empty]" }
                } @else {
                    @if let Some(edit_item) = edit_item {
                        form
                            name="edit-item"
                            id="edit-item"
                            action=(format!("/inventory/item/{edit_item}/edit"))
                            target="_self"
                            method="post"
                        {}
                    }
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
                                @if edit_item.map_or(false, |edit_item| edit_item == item.id) {
                                    tr ."h-10" {
                                        td ."border" ."p-2" ."bg-blue-100" {
                                            input ."w-full"
                                                type="text"
                                                id="edit-item-name"
                                                name="edit-item-name"
                                                form="edit-item"
                                                value=(item.name)
                                            {}
                                        }
                                        td ."border" ."p-2" ."bg-blue-100" {
                                            input ."w-full"
                                                type="number"
                                                id="edit-item-weight"
                                                name="edit-item-weight"
                                                form="edit-item"
                                                value=(item.weight)
                                            {}
                                        }
                                        td ."border" ."p-2" ."bg-green-100" {
                                            button type="submit" form="edit-item" {
                                                span ."mdi" ."mdi-content-save" ."text-xl" {}
                                            }
                                        }
                                        td ."border" ."p-2" ."bg-red-100" {
                                            a href=(format!("/inventory/item/{id}/cancel", id = item.id)) {
                                                span ."mdi" ."mdi-cancel" ."text-xl" {}
                                            }
                                        }
                                    }
                                } @else {
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
                                            right:0;", width=(f64::from(item.weight) / f64::from(biggest_item_weight) * 100.0))) {}
                                        }
                                        td
                                            ."border"
                                            ."bg-blue-200"
                                            ."hover:bg-blue-400"
                                            ."cursor-pointer"
                                            ."w-8"
                                            ."text-center"
                                            {
                                                a href = (format!("?edit_item={id}", id = item.id))
                                                {
                                                    button {
                                                        span ."mdi" ."mdi-pencil" ."text-xl" {}
                                                    }
                                                }
                                        }
                                        td
                                            ."border"
                                            ."bg-red-200"
                                            ."hover:bg-red-400"
                                            ."cursor-pointer"
                                            ."w-8"
                                            ."text-center"
                                            {
                                                a href = (format!("/inventory/item/{id}/delete", id = item.id))
                                                {
                                                    button {
                                                        span ."mdi" ."mdi-delete" ."text-xl" {}
                                                    }
                                                }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        );

        Self { doc }
    }
}

impl From<InventoryItemList> for Markup {
    fn from(val: InventoryItemList) -> Self {
        val.doc
    }
}

pub struct InventoryNewItemForm {
    doc: Markup,
}

impl InventoryNewItemForm {
    pub fn build(state: &ClientState, categories: &Vec<Category>) -> Self {
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
                            label for="name" .font-bold ."w-1/2" ."p-2" ."text-center" { "Name" }
                            span ."w-1/2" {
                                input type="text" id="new-item-name" name="new-item-name"
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
                        label for="weight" .font-bold ."w-1/2" .text-center { "Weight" }
                        span ."w-1/2" {
                            input
                                type="text"
                                id="new-item-weight"
                                name="new-item-weight"
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
                                    id="new-item-category-id"
                                    name="new-item-category-id"
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

        Self { doc }
    }
}

impl From<InventoryNewItemForm> for Markup {
    fn from(val: InventoryNewItemForm) -> Self {
        val.doc
    }
}
// impl InventoryItemList {
//     pub fn to_string(self) -> String {
//         self.doc.into_string()
//     }
// }
//ItemList
