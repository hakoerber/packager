use maud::{html, Markup};

use crate::elements::{
    self,
    list::{self, Action, List},
};
use crate::ClientState;

use super::model;
use uuid::Uuid;

pub struct Inventory;

impl Inventory {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory",
        fields(component = "Inventory")
        skip(categories)
    )]
    pub fn build(
        active_category: Option<&model::Category>,
        categories: &Vec<model::Category>,
        edit_item_id: Option<Uuid>,
    ) -> Markup {
        html!(
            div id="pkglist-item-manager" {
                div ."p-8" ."grid" ."grid-cols-4" ."gap-5" {
                    div ."col-span-2" ."flex" ."flex-col" ."gap-8" {
                        h1 ."text-2xl" ."text-center" { "Categories" }
                        (InventoryCategoryList::build(active_category, categories))
                        (InventoryNewCategoryForm::build())
                    }
                    div ."col-span-2" ."flex" ."flex-col" ."gap-8" {
                        h1 ."text-2xl" ."text-center" { "Items" }
                        @if let Some(active_category) = active_category {
                            (InventoryItemList::build(edit_item_id, active_category.items()))
                        }
                        (InventoryNewItemForm::build(active_category, categories))
                    }
                }
            }
        )
    }
}

pub struct InventoryCategoryList;

impl InventoryCategoryList {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_category_list",
        fields(component = "InventoryCategoryList"),
        skip(categories)
    )]
    pub fn build(
        active_category: Option<&model::Category>,
        categories: &[model::Category],
    ) -> Markup {
        let biggest_category_weight: i32 = categories
            .iter()
            .map(model::Category::total_weight)
            .max()
            .unwrap_or(1);

        struct Row<'a> {
            category: &'a model::Category,
            active: bool,
            biggest_category_weight: i32,
        }
        impl<'a> list::Row for Row<'a> {
            fn is_active(&self) -> bool {
                self.active
            }

            fn cells(&self) -> Vec<list::Cell> {
                vec![
                    list::Cell {
                        cell_type: list::CellType::Link(list::Link {
                            text: &self.category.name,
                            href: format!("/inventory/category/{}", self.category.id),
                            hx_config: Some(elements::HxConfig {
                                hx_post: format!(
                                    "/inventory/categories/{}/select",
                                    self.category.id
                                ),
                                hx_swap: elements::HxSwap::OuterHtml,
                                hx_target: "#pkglist-item-manager",
                            }),
                        }),
                    },
                    list::Cell {
                        cell_type: list::CellType::NumberWithBar(list::NumberWithBar {
                            value: self.category.total_weight(),
                            max_value: self.biggest_category_weight,
                        }),
                    },
                ]
            }
        }

        List {
            id: Some("category-list"),
            editing_config: None,
            header: list::Header {
                cells: vec![
                    Some(list::HeaderCell { title: "Name" }),
                    Some(list::HeaderCell { title: "Weight" }),
                ],
            },
            rows: categories
                .iter()
                .map(|category| {
                    let active = active_category.map_or(false, |c| category.id == c.id);
                    Row {
                        category,
                        active,
                        biggest_category_weight,
                    }
                })
                .collect(),
        }
        .render()
    }
}

pub struct InventoryItemList;

impl InventoryItemList {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_item_list",
        fields(component = "InventoryItemList"),
        skip(items)
    )]
    pub fn build(edit_item_id: Option<Uuid>, items: &Vec<model::Item>) -> Markup {
        let biggest_item_weight: i32 = items.iter().map(|item| item.weight).max().unwrap_or(1);

        struct Row<'a> {
            item: &'a model::Item,
            biggest_item_weight: i32,
            edit_item_id: Option<Uuid>,
        }

        impl<'a> list::Row for Row<'a> {
            fn cells(&self) -> Vec<list::Cell> {
                vec![
                    list::Cell {
                        cell_type: list::CellType::Link(list::Link {
                            text: &self.item.name,
                            href: format!("/inventory/item/{id}/", id = self.item.id),
                            hx_config: None,
                        }),
                    },
                    list::Cell {
                        cell_type: list::CellType::NumberWithBar(list::NumberWithBar {
                            value: self.item.weight,
                            max_value: self.biggest_item_weight,
                        }),
                    },
                ]
            }

            fn is_edit(&self) -> bool {
                self.edit_item_id.map_or(false, |id| id == self.item.id)
            }
        }

        let table = list::List {
            id: None,
            editing_config: Some(Box::new(|row: Row| list::EditingConfig {
                edit_action: Action::Href(format!("?edit_item={id}", id = row.item.id)),
                edit_hx_config: None,

                delete_action: Action::Href(format!(
                    "/inventory/item/{id}/delete",
                    id = row.item.id
                )),
                delete_hx_config: None,

                save_action: Action::Submit("edit-item"),
                save_hx_config: None,

                cancel_action: Action::Href(format!(
                    "/inventory/item/{id}/cancel",
                    id = row.item.id
                )),
                cancel_hx_config: None,
            })),
            header: list::Header {
                cells: vec![
                    Some(list::HeaderCell { title: "Name" }),
                    Some(list::HeaderCell { title: "Weight" }),
                ],
            },
            rows: items
                .iter()
                .map(|item| Row {
                    item,
                    biggest_item_weight,
                    edit_item_id,
                })
                .collect(),
        };

        html!(
            div #items {
                @if items.is_empty() {
                    p ."text-lg" ."text-center" ."py-5" ."text-gray-400" { "[Empty]" }
                } @else {
                    @if let Some(edit_item_id) = edit_item_id {
                        form
                            name="edit-item"
                            id="edit-item"
                            action={"/inventory/item/" (edit_item_id) "/edit"}
                            target="_self"
                            method="post"
                        {}
                    }
                    (table.render())
                    table
                        ."table"
                        ."table-auto"
                        ."table-fixed"
                        ."border-collapse"
                        ."border-spacing-0"
                        ."border"
                        ."w-full"
                    {
                        thead ."bg-gray-200" {
                            tr ."h-10" {
                                th ."border" ."p-2" ."w-3/5" { "Name" }
                                th ."border" ."p-2" { "Weight" }
                                th ."border" ."p-2" ."w-10" {}
                                th ."border" ."p-2" ."w-10" {}
                            }
                        }
                        tbody {
                            @for item in items {
                                @if edit_item_id.map_or(false, |id| id == item.id) {
                                    tr ."h-10" {
                                        td ."border" ."bg-blue-300" ."px-2" ."py-0" {
                                            div ."h-full" ."w-full" ."flex" {
                                                input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                                                    type="text"
                                                    id="edit-item-name"
                                                    name="edit-item-name"
                                                    form="edit-item"
                                                    value=(item.name)
                                                {}
                                            }
                                        }
                                        td ."border" ."bg-blue-300" ."px-2" ."py-0" {
                                            div ."h-full" ."w-full" ."flex" {
                                                input ."m-auto" ."px-1"."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                                                    type="number"
                                                    id="edit-item-weight"
                                                    name="edit-item-weight"
                                                    form="edit-item"
                                                    value=(item.weight)
                                                {}
                                            }
                                        }
                                        td
                                            ."border-none"
                                            ."bg-green-100"
                                            ."hover:bg-green-200"
                                            ."p-0"
                                            ."h-full"
                                        {
                                            button
                                                ."aspect-square"
                                                ."flex"
                                                ."w-full"
                                                ."h-full"
                                                type="submit"
                                                form="edit-item"
                                            {
                                                span
                                                    ."m-auto"
                                                    ."mdi"
                                                    ."mdi-content-save"
                                                    ."text-xl"
                                                {}
                                            }
                                        }
                                        td
                                            ."border-none"
                                            ."bg-red-100"
                                            ."hover:bg-red-200"
                                            ."p-0"
                                            ."h-full"
                                        {
                                            a
                                                href=(format!("/inventory/item/{id}/cancel", id = item.id))
                                                ."aspect-square"
                                                ."flex"
                                                ."w-full"
                                                ."h-full"
                                                ."p-0"
                                            {
                                                span
                                                    ."m-auto"
                                                    ."mdi"
                                                    ."mdi-cancel"
                                                    ."text-xl"
                                                {}
                                            }
                                        }
                                    }
                                } @else {
                                    tr ."h-10" {
                                        td ."border" ."p-0" {
                                            a
                                                ."p-2" ."w-full" ."inline-block"
                                                href=(
                                                    format!("/inventory/item/{id}/", id=item.id)
                                                )
                                            {
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
                                            ."border-none"
                                            ."p-0"
                                            ."bg-blue-200"
                                            ."hover:bg-blue-400"
                                            ."w-8"
                                            ."h-full"
                                            {
                                                a
                                                    href=(format!("?edit_item={id}", id = item.id))
                                                    ."aspect-square"
                                                    ."flex"
                                                    ."w-full"
                                                {
                                                    span ."m-auto" ."mdi" ."mdi-pencil" ."text-xl" {}
                                                }
                                        }
                                        td
                                            ."border-none"
                                            ."p-0"
                                            ."bg-red-200"
                                            ."hover:bg-red-400"
                                            ."w-8"
                                            ."h-full"
                                        {
                                            a
                                                href=(format!("/inventory/item/{id}/delete", id = item.id))
                                                ."aspect-square"
                                                ."flex"
                                                ."w-full"
                                            {
                                                span ."m-auto" ."mdi" ."mdi-delete" ."text-xl" {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        )
    }
}

pub struct InventoryNewItemFormName;

impl InventoryNewItemFormName {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_new_item_form_name",
        fields(component = "InventoryNewItemFormName")
    )]
    pub fn build(value: Option<&str>, error: bool) -> Markup {
        html!(
            div
                ."grid"
                ."grid-cols-[2fr,3fr]"
                ."justify-items-center"
                ."items-center"
                hx-post="/inventory/item/name/validate"
                hx-trigger="input delay:1s, loaded from:document"
                hx-target="this"
                hx-params="new-item-name"
                hx-swap="outerHTML"
            {
                label for="name" .font-bold { "Name" }
                input
                    type="text"
                    id="new-item-name"
                    name="new-item-name"
                    x-on:input="(e) => {save_active = inventory_new_item_check_input()}"
                    ."block"
                    ."w-full"
                    ."p-2"
                    ."bg-gray-50"
                    ."border-2"
                    ."border-red-500"[error]
                    ."border-gray-300"[!error]
                    ."focus:outline-none"
                    ."focus:bg-white"
                    ."focus:border-gray-500"[!error]
                    value=[value]
                {}
                @if error {
                    div
                        ."col-start-2"
                        ."text-sm"
                        ."text-red-500"
                    { "name already exists" }
                }
            }
        )
    }
}

pub struct InventoryNewItemFormWeight;

impl InventoryNewItemFormWeight {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_new_item_form_weight",
        fields(component = "InventoryNewItemFormWeight")
    )]
    pub fn build() -> Markup {
        html!(
            div
                ."grid"
                ."grid-cols-[2fr,3fr]"
                ."justify-items-center"
                ."items-center"
            {
                label for="weight" .font-bold { "Weight" }
                input
                    type="number"
                    id="new-item-weight"
                    name="new-item-weight"
                    min="0"
                    x-on:input="(e) => {
                        save_active = inventory_new_item_check_input();
                        weight_error = !check_weight();
                    }"
                    x-bind:class="weight_error && 'border-red-500' || 'border-gray-300 focus:border-gray-500'"
                    ."block"
                    ."w-full"
                    ."p-2"
                    ."bg-gray-50"
                    ."border-2"
                    ."focus:outline-none"
                    ."focus:bg-white"
                {}
                span
                    // x-on produces some errors, this works as well
                    x-bind:class="!weight_error && 'hidden'"
                    ."col-start-2"
                    ."text-sm"
                    ."text-red-500"
                { "invalid input" }
            }
        )
    }
}

pub struct InventoryNewItemFormCategory;

impl InventoryNewItemFormCategory {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_new_item_form_category",
        fields(component = "InventoryNewItemFormCategory"),
        skip(categories)
    )]
    pub fn build(
        active_category: Option<&model::Category>,
        categories: &Vec<model::Category>,
    ) -> Markup {
        html!(
            div
                ."grid"
                ."grid-cols-[2fr,3fr]"
                ."justify-items-center"
                ."items-center"
            {
                label for="item-category" .font-bold ."w-1/2" .text-center { "Category" }
                select
                        id="new-item-category-id"
                        name="new-item-category-id"
                        ."block"
                        ."w-full"
                        ."p-2"
                        ."bg-gray-50"
                        ."border-2"
                        ."border-gray-300"
                        ."focus:outline-none"
                        ."focus:bg-white"
                        ."focus:border-gray-500"
                        autocomplete="off" // https://stackoverflow.com/a/10096033
                    {
                    @for category in categories {
                        option value=(category.id) selected[active_category.map_or(false, |c| c.id == category.id)] {
                            (category.name)
                        }
                    }
                }
            }
        )
    }
}

pub struct InventoryNewItemForm;

impl InventoryNewItemForm {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_new_item_form",
        fields(component = "InventoryNewItemForm"),
        skip(categories)
    )]
    pub fn build(
        active_category: Option<&model::Category>,
        categories: &Vec<model::Category>,
    ) -> Markup {
        html!(
            form
                x-data="{
                    save_active: inventory_new_item_check_input(),
                    weight_error: !check_weight(),
                }"
                name="new-item"
                hx-post="/inventory/item/"
                hx-swap="outerHTML"
                hx-target="#pkglist-item-manager"
                id="new-item"
                action="/inventory/item/"
                target="_self"
                method="post"
                ."p-5" ."border-2" ."border-gray-200" {
                div ."mb-5" ."flex" ."flex-row" ."items-center" {
                    span ."mdi" ."mdi-playlist-plus" ."text-2xl" ."mr-4" {}
                    p ."inline" ."text-xl" { "Add new item" }
                }
                div ."w-11/12" ."mx-auto" ."flex" ."flex-col" ."gap-8" {
                    (InventoryNewItemFormName::build(None, false))
                    (InventoryNewItemFormWeight::build())
                    (InventoryNewItemFormCategory::build(active_category, categories))
                    input type="submit" value="Add"
                        x-bind:disabled="!save_active"
                        ."enabled:cursor-pointer"
                        ."disabled:opacity-50"
                        ."py-2"
                        ."border-2"
                        ."border-gray-300"
                        ."mx-auto"
                        ."w-full" {
                    }
                }
            }
        )
    }
}

pub struct InventoryNewCategoryForm;

impl InventoryNewCategoryForm {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_new_category_form",
        fields(component = "InventoryNewCategoryForm")
    )]
    pub fn build() -> Markup {
        html!(
            form
                x-data="{ save_active: document.getElementById('new-category-name').value.length != 0 }"
                name="new-category"
                id="new-category"
                action="/inventory/category/"
                target="_self"
                method="post"
                ."p-5" ."border-2" ."border-gray-200" {
                div ."mb-5" ."flex" ."flex-row" ."items-center" {
                    span ."mdi" ."mdi-playlist-plus" ."text-2xl" ."mr-4" {}
                    p ."inline" ."text-xl" { "Add new category" }
                }
                div ."w-11/12" ."mx-auto" {
                    div ."pb-8" {
                        div ."flex" ."flex-row" ."justify-center" ."items-start"{
                            label for="name" .font-bold ."w-1/2" ."p-2" ."text-center" { "Name" }
                            span ."w-1/2" {
                                input type="text" id="new-category-name" name="new-category-name"
                                    x-on:input="(e) => {save_active = e.target.value.length != 0 }"
                                    ."block"
                                    ."w-full"
                                    ."p-2"
                                    ."bg-gray-50"
                                    ."border-2"
                                    ."border-gray-300"
                                    ."focus:outline-none"
                                    ."focus:bg-white"
                                    ."focus:border-gray-500"
                                    {
                                    }
                            }
                        }
                    }
                    input type="submit" value="Add"
                        x-bind:disabled="!save_active"
                        ."enabled:cursor-pointer"
                        ."disabled:opacity-50"
                        ."py-2"
                        ."border-2"
                        ."border-gray-300"
                        ."mx-auto"
                        ."w-full" {
                    }
                }
            }
        )
    }
}

pub struct InventoryItem;

impl InventoryItem {
    #[tracing::instrument(
        target = "packager::html::build",
        name = "build_inventory_item",
        fields(component = "InventoryItem")
    )]
    pub fn build(_state: &ClientState, item: &model::InventoryItem) -> Markup {
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
                            td ."border" ."p-2" { (item.name) }
                        }
                        tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                            td ."border" ."p-2" { "Description" }
                            td ."border" ."p-2" { (item.description.clone().unwrap_or(String::new())) }
                        }
                        tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                            td ."border" ."p-2" { "Weight" }
                            td ."border" ."p-2" { (item.weight.to_string()) }
                        }
                        tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                            td ."border" ."p-2" { "Category" }
                            td ."border" ."p-2" { (item.category.name) }
                        }
                    }
                }
                @match item.product {
                    Some(ref product) => p { "this item is part of product" (product.name) },
                    None => p { "this item is not part of a product" },
                }
            }
        )
    }
}
