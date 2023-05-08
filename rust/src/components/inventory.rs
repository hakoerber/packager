use super::Tree;
use axohtml::{
    html, text,
    types::{Class, SpacedSet},
};

use crate::models::*;
use crate::State;

pub struct Inventory {
    doc: Tree,
}

impl Inventory {
    pub async fn build(state: State, categories: Vec<Category>) -> Result<Self, Error> {
        let doc = html!(
            <div id="pkglist-item-manager">
                <div class=["p-8", "grid", "grid-cols-4", "gap-3"]>
                    <div class=["col-span-2"]>
                        {<InventoryCategoryList as Into<Tree>>::into(InventoryCategoryList::build(&categories).await?)}
                    </div>
                    {if state.has_active_category { html!(
                        <div class=["col-span-2"]>
                            {<InventoryItemList as Into<Tree>>::into(InventoryItemList::build(categories.iter().find(|category| category.active).unwrap().items()).await?)}
                        </div>
                    )} else {
                        html!(<div></div>)
                    }}
                </div>
            </div>
        );

        Ok(Self { doc })
    }
}

impl Into<Tree> for Inventory {
    fn into(self) -> Tree {
        self.doc
    }
}

pub struct InventoryCategoryList {
    doc: Tree,
}

impl InventoryCategoryList {
    pub async fn build(categories: &Vec<Category>) -> Result<Self, Error> {
        let biggest_category_weight: u32 = categories
            .iter()
            .map(|category| category.total_weight())
            .max()
            .unwrap_or(1);

        let cls_td_active: SpacedSet<Class> =
            ["border", "p-0", "m-0", "font-bold"].try_into().unwrap();
        let cls_td_inactive: SpacedSet<Class> = ["border", "p-0", "m-0"].try_into().unwrap();

        let cls_tr_active: SpacedSet<Class> = [
            "h-10",
            "hover:bg-purple-100",
            "m-3",
            "h-full",
            "outline",
            "outline-2",
            "outline-indigo-600",
        ]
        .try_into()
        .unwrap();
        let cls_tr_inactive: SpacedSet<Class> = ["h-10", "hover:bg-purple-100", "m-3", "h-full"]
            .try_into()
            .unwrap();

        let doc = html!(
            <div>
                <h1 class=["text-2xl", "mb-5"]>"Categories"</h1>
                <table class=[
                    "table",
                    "table-auto",
                    "border-collapse",
                    "border-spacing-0",
                    "border",
                    "w-full",
                ]>

                      <colgroup>
                            <col style="width:50%"/>
                            <col style="width:50%"/>
                      </colgroup>
                    <thead class=["bg-gray-200"]>
                        <tr class=["h-10"]>
                            <th class=["border", "p-2"]>"Name"</th>
                            <th class=["border", "p-2"]>"Weight"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {categories.iter().map(|category| html!(
                            <tr
                                class={if category.active {
                                        cls_tr_active.clone()
                                    } else {
                                        cls_tr_inactive.clone()
                                    }}
                                >
                                <td
                                    class={if category.active {
                                            cls_td_active.clone()
                                        } else {
                                            cls_td_inactive.clone()
                                        }}
                                    >
                                    <a
                                        id="select-category"
                                        href={
                                            format!(
                                                "/inventory/category/{id}",
                                                id=category.id
                                            )
                                        }
                                        class=["inline-block", "p-2", "m-0", "w-full"]
                                    >
                                        {text!(category.name.clone())}
                                    </a>
                                </td>
                                <td class=["border", "p-0", "m-0"] style="position:relative;">
                                    <a
                                        id="select-category"
                                        href={
                                            format!(
                                                "/inventory/category/{id}",
                                                id=category.id
                                            )
                                        }
                                        class=["inline-block", "p-2", "m-0", "w-full"]
                                    >
                                        <p>
                                            {text!(category.total_weight().to_string())}
                                        </p>
                                    </a>
                                    <div
                                        class=["bg-blue-600", "h-1.5"]
                                        style = {
                                            format!(
                                                "width: {width}%;position:absolute;left:0;bottom:0;right:0;",
                                                width=(
                                                    category.total_weight() as f32
                                                    / biggest_category_weight as f32
                                                    * 100.0
                                                )
                                            )
                                        }
                                        >
                                    </div>
                                </td>
                            </tr>
                        ))}
                        <tr class=["h-10", "hover:bg-purple-200", "bg-gray-300", "font-bold"]>
                            <td class=["border", "p-0", "m-0"]>
                                <p class=["p-2", "m-2"]>"Sum"</p>
                            </td>
                            <td class=["border", "p-0", "m-0"]>
                                <p class=["p-2", "m-2"]>
                                    {text!(categories.iter().map(|category| category.total_weight()).sum::<u32>().to_string())}
                                </p>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        );

        Ok(Self { doc })
    }
}

impl Into<Tree> for InventoryCategoryList {
    fn into(self) -> Tree {
        self.doc
    }
}

pub struct InventoryItemList {
    doc: Tree,
}

impl InventoryItemList {
    pub async fn build(items: &Vec<Item>) -> Result<Self, Error> {
        let doc = html!(
            <div>
                <h1 class=["text-2xl", "mb-5"]>"Categories"</h1>
                <table class=[
                    "table",
                    "table-auto",
                    "border-collapse",
                    "border-spacing-0",
                    "border",
                    "w-full",
                ]>
                    <thead class=["bg-gray-200"]>
                        <tr class=["h-10"]>
                            <th class=["border", "p-2"]>"Name"</th>
                            <th class=["border", "p-2"]>"Weight"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {items.iter().map(|item| html!(
                            <tr class=["h-10", "even:bg-gray-100", "hover:bg-purple-100"]>
                                <td class=["border", "p-0", "m-0"]>
                                    <a
                                        class=["p-2", "w-full", "inline-block"]
                                        href={
                                            format!("/inventory/item/{id}/", id=item.id)
                                        }
                                        >
                                    {text!(item.name.clone())}
                                    </a>
                                </td>
                                <td class=["border", "p-0", "m-0"]>
                                    {text!(item.weight.to_string())}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        );

        Ok(Self { doc })
    }
}

impl Into<Tree> for InventoryItemList {
    fn into(self) -> Tree {
        self.doc
    }
}
