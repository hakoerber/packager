use maud::{html, Markup};
use uuid::Uuid;

use crate::models;

pub struct TripPackageListRowReady;

impl TripPackageListRowReady {
    pub fn build(trip_id: Uuid, item: &models::trips::TripItem) -> Markup {
        html!(
            li
                ."flex"
                ."flex-row"
                ."justify-between"
                ."items-stretch"
                ."bg-green-50"[item.packed]
                ."bg-red-50"[!item.packed]
                ."hover:bg-white"[!item.packed]
                ."h-full"
            {
                span
                    ."p-2"
                {
                    (item.item.name)
                }
                @if item.packed {
                    a
                        href={
                            "/trips/" (trip_id)
                            "/items/" (item.item.id)
                            "/unpack"
                        }
                        hx-post={
                            "/trips/" (trip_id)
                            "/packagelist/item/"
                            (item.item.id) "/unpack"
                        }
                        hx-target="closest li"
                        hx-swap="outerHTML"
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                    {
                        span
                            ."mdi"
                            ."m-auto"
                            ."text-xl"
                            ."mdi-check"
                        {}
                    }
                } @else {
                    a
                        href={
                            "/trips/" (trip_id)
                            "/items/" (item.item.id)
                            "/pack"
                        }
                        hx-post={
                            "/trips/" (trip_id)
                            "/packagelist/item/"
                            (item.item.id) "/pack"
                        }
                        hx-target="closest li"
                        hx-swap="outerHTML"
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                    {
                        span
                            ."mdi"
                            ."m-auto"
                            ."text-xl"
                            ."mdi-checkbox-blank-outline"
                        {}
                    }
                }
            }
        )
    }
}

pub struct TripPackageListRowUnready;

impl TripPackageListRowUnready {
    pub fn build(trip_id: Uuid, item: &models::trips::TripItem) -> Markup {
        html!(
            li
                ."flex"
                ."flex-row"
                ."justify-between"
                ."items-stretch"
                ."bg-green-50"[item.ready]
                ."bg-red-50"[!item.ready]
                ."hover:bg-white"[!item.ready]
                ."h-full"
            {
                span
                    ."p-2"
                {
                    (item.item.name)
                }
                @if item.ready {
                    a
                        href={
                            "/trips/" (trip_id)
                            "/items/" (item.item.id)
                            "/unready"
                        }
                        hx-post={
                            "/trips/" (trip_id)
                            "/packagelist/item/"
                            (item.item.id) "/unready"
                        }
                        hx-target="closest li"
                        hx-swap="outerHTML"
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                    {
                        span
                            ."mdi"
                            ."m-auto"
                            ."text-xl"
                            ."mdi-check"
                        {}
                    }
                } @else {
                    a
                        href={
                            "/trips/" (trip_id)
                            "/items/" (item.item.id)
                            "/ready"
                        }
                        hx-post={
                            "/trips/" (trip_id)
                            "/packagelist/item/"
                            (item.item.id) "/ready"
                        }
                        hx-target="closest li"
                        hx-swap="outerHTML"
                        ."flex"
                        ."flex-row"
                        ."aspect-square"
                    {
                        span
                            ."mdi"
                            ."m-auto"
                            ."text-xl"
                            ."mdi-checkbox-blank-outline"
                        {}
                    }
                }
            }
        )
    }
}

pub struct TripPackageListCategoryBlockReady;

impl TripPackageListCategoryBlockReady {
    pub fn build(trip: &models::trips::Trip, category: &models::trips::TripCategory) -> Markup {
        let empty = !category
            .items
            .as_ref()
            .unwrap()
            .iter()
            .any(|item| item.picked);

        html!(
            div
                ."inline-block"
                ."w-full"
                ."mb-5"
                ."border"
                ."border-2"
                ."border-gray-300"
                ."opacity-30"[empty]
            {
                div
                    ."bg-gray-100"
                    ."border-b-2"
                    ."border-gray-300"
                    ."p-3"
                {
                    h3 { (category.category.name) }
                }
                @if empty {
                    div
                        ."flex"
                        ."p-1"
                    {
                        span
                            ."text-sm"
                            ."m-auto"
                        {
                            "no items picked"
                        }
                    }
                } @else {
                    ul
                        ."flex"
                        ."flex-col"
                    {
                        @for item in category.items.as_ref().unwrap().iter().filter(|item| item.picked) {
                            (TripPackageListRowReady::build(trip.id, item))
                        }
                    }
                }
            }
        )
    }
}

pub struct TripPackageListCategoryBlockUnready;

impl TripPackageListCategoryBlockUnready {
    pub fn build(trip: &models::trips::Trip, category: &models::trips::TripCategory) -> Markup {
        let empty = !category
            .items
            .as_ref()
            .unwrap()
            .iter()
            .any(|item| item.picked);

        html!(
            div
                ."inline-block"
                ."w-full"
                ."mb-5"
                ."border"
                ."border-2"
                ."border-gray-300"
                ."opacity-30"[empty]
            {
                div
                    ."bg-gray-100"
                    ."border-b-2"
                    ."border-gray-300"
                    ."p-3"
                {
                    h3 { (category.category.name) }
                }
                @if empty {
                    div
                        ."flex"
                        ."p-1"
                    {
                        span
                            ."text-sm"
                            ."m-auto"
                        {
                            "no items picked"
                        }
                    }
                } @else {
                    ul
                        ."flex"
                        ."flex-col"
                    {
                        @for item in category.items.as_ref().unwrap().iter().filter(|item| item.picked) {
                            (TripPackageListRowUnready::build(trip.id, item))
                        }
                    }
                }
            }
        )
    }
}
pub struct TripPackageList;

impl TripPackageList {
    pub fn build(trip: &models::trips::Trip) -> Markup {
        // let all_packed = trip.categories().iter().all(|category| {
        //     category
        //         .items
        //         .as_ref()
        //         .unwrap()
        //         .iter()
        //         .all(|item| !item.picked || item.packed)
        // });
        let has_unready_items: bool = trip.categories.as_ref().unwrap().iter().any(|category| {
            category
                .items
                .as_ref()
                .unwrap()
                .iter()
                .any(|item| item.picked && !item.ready)
        });
        html!(
            div
                ."p-8"
                ."flex"
                ."flex-col"
                ."gap-8"
            {
                div
                    ."flex"
                    ."flex-row"
                    ."justify-between"
                {
                    h1 ."text-xl" {
                        "Package list for "
                        a
                            href={"/trips/" (trip.id) "/"}
                            hx-boost="true"
                            ."font-bold"
                        {
                            (trip.name)
                        }
                    }
                    a
                        href={"/trips/" (trip.id) "/packagelist/"}
                        hx-boost="true"
                        // disabled[!all_packed]
                        // ."opacity-50"[!all_packed]
                        ."p-2"
                        ."border-2"
                        ."border-gray-500"
                        ."bg-blue-200"
                        ."hover:bg-blue-200"
                    {
                        "Finish packing"
                    }
                }
                @if has_unready_items {
                    p { "There are items that are not yet ready, get them!"}
                    div
                        ."columns-3"
                        ."gap-5"
                    {
                        @for category in trip.categories() {
                            (TripPackageListCategoryBlockUnready::build(trip, category))
                        }
                    }
                } @else {
                    p { "All items are ready, pack the following things:" }
                    div
                        ."columns-3"
                        ."gap-5"
                    {
                        @for category in trip.categories() {
                            (TripPackageListCategoryBlockReady::build(trip, category))
                        }
                    }
                }
            }
        )
    }
}
