use crate::models;
use crate::models::*;

use maud::{html, Markup};

pub struct TripManager {
    doc: Markup,
}

impl TripManager {
    pub fn build(trips: Vec<models::Trip>) -> Self {
        let doc = html!(
            div ."p-8" {
                (TripTable::build(trips).into_markup())
                (NewTrip::build().into_markup())
            }
        );

        Self { doc }
    }
}

pub struct TripTable {
    doc: Markup,
}

impl From<TripManager> for Markup {
    fn from(val: TripManager) -> Self {
        val.doc
    }
}

impl TripTable {
    pub fn build(trips: Vec<models::Trip>) -> Self {
        let doc = html!(
            h1 ."text-2xl" ."mb-5" {"Trips"}
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
                        th ."border" ."p-2" { "From" }
                        th ."border" ."p-2" { "To" }
                        th ."border" ."p-2" { "Nights" }
                        th ."border" ."p-2" { "State" }
                    }
                }
                tbody {
                    @for trip in trips {
                        tr ."h-10" ."even:bg-gray-100" ."hover:bg-purple-100" ."h-full" {
                            td ."border" ."p-0" ."m-0" {
                                a ."inline-block" ."p-2" ."m-0" ."w-full"
                                    href=(format!("/trip/{id}/", id=trip.id))
                                { (trip.name) }
                            }
                            td ."border" ."p-0" ."m-0" {
                                a ."inline-block" ."p-2" ."m-0" ."w-full"
                                    href=(format!("/trip/{id}/", id=trip.id))
                                { (trip.start_date) }
                            }
                            td ."border" ."p-0" ."m-0" {
                                a ."inline-block" ."p-2" ."m-0" ."w-full"
                                    href=(format!("/trip/{id}/", id=trip.id))
                                { (trip.end_date) }
                            }
                            td ."border" ."p-0" ."m-0" {
                                a ."inline-block" ."p-2" ."m-0" ."w-full"
                                    href=(format!("/trip/{id}/", id=trip.id))
                                { ((trip.end_date - trip.start_date).whole_days()) }
                            }
                            td ."border" ."p-0" ."m-0" {
                                a ."inline-block" ."p-2" ."m-0" ."w-full"
                                    href=(format!("/trip/{id}/", id=trip.id))
                                { (trip.state.to_string()) }
                            }
                        }
                    }
                }
            }
        );

        Self { doc }
    }

    pub fn into_markup(self) -> Markup {
        self.doc
    }
}

pub struct NewTrip {
    doc: Markup,
}

impl NewTrip {
    pub fn build() -> Self {
        let doc = html!(
            form
                name="new_trip"
                action="/trip/"
                target="_self"
                method="post"
                ."mt-8" ."p-5" ."border-2" ."border-gray-200"
            {
                div ."mb-5" ."flex" ."flex-row" ."trips-center" {
                    span ."mdi" ."mdi-playlist-plus" ."text-2xl" ."mr-4" {}
                    p ."inline" ."text-xl" { "Add new trip" }
                }
                div ."w-11/12" ."m-auto" {
                    div ."mx-auto" ."pb-8" {
                        div ."flex" ."flex-row" ."justify-center" {
                            label for="trip-name" ."font-bold" ."w-1/2" ."p-2" ."text-center" { "Name" }
                            span ."w-1/2" {
                                input
                                    type="text"
                                    id="trip-name"
                                    name="new-trip-name"
                                    ."block"
                                    ."w-full"
                                    ."p-2"
                                    ."bg-gray-50"
                                    ."border-2"
                                    ."rounded"
                                    ."focus:outline-none"
                                    ."focus:bg-white"
                                {}
                            }
                        }
                    }
                    div ."mx-auto" ."pb-8" {
                        div ."flex" ."flex-row" ."justify-center" {
                            label for="trip-name" ."font-bold" ."w-1/2" ."p-2" ."text-center" { "Start date" }
                            span ."w-1/2" {
                                input
                                    type="date"
                                    id="start-date"
                                    name="new-trip-start-date"
                                    ."block"
                                    ."w-full"
                                    ."p-2"
                                    ."bg-gray-50"
                                    ."appearance-none"
                                    ."border-2"
                                    ."border-gray-300"
                                    ."rounded"
                                    ."focus:outline-none"
                                    ."focus:bg-white"
                                    ."focus:border-purple-500"
                                {}
                            }
                        }
                    }
                    div ."mx-auto" ."pb-8" {
                        div ."flex" ."flex-row" ."justify-center" {
                            label for="trip-name" ."font-bold" ."w-1/2" ."p-2" ."text-center" { "Start date" }
                            span ."w-1/2" {
                                input
                                    type="date"
                                    id="end-date"
                                    name="new-trip-end-date"
                                    ."block"
                                    ."w-full"
                                    ."p-2"
                                    ."bg-gray-50"
                                    ."appearance-none"
                                    ."border-2"
                                    ."border-gray-300"
                                    ."rounded"
                                    ."focus:outline-none"
                                    ."focus:bg-white"
                                    ."focus:border-purple-500"
                                {}
                            }
                        }
                    }
                    input
                        type="submit"
                        value="Add"
                        ."py-2"
                        ."border-2"
                        ."rounded"
                        ."border-gray-300"
                        ."mx-auto"
                        ."w-full"
                    {}
                }
            }
        );

        Self { doc }
    }

    pub fn into_markup(self) -> Markup {
        self.doc
    }
}

pub struct Trip {
    doc: Markup,
}

impl Trip {
    pub fn build(trip: &models::Trip) -> Self {
        let doc = html!(
            div ."p-8" {
                div ."flex" ."flex-row" ."items-center" ."gap-x-3" {
                    h1 ."text-2xl" ."font-semibold"{ (trip.name) }
                }
                div ."my-6" {
                    (TripInfo::build(&trip).into_markup())
                }
            }
        );

        Self { doc }
    }

    pub fn into_markup(self) -> Markup {
        self.doc
    }
}

pub struct TripInfo {
    doc: Markup,
}

impl TripInfo {
    pub fn build(trip: &models::Trip) -> Self {
        let doc = html!(
            table
                ."table"
                ."table-auto"
                ."border-collapse"
                ."border-spacing-0"
                ."border"
                ."w-full"
            {
                tbody {
                    tr {
                        td ."border" ."p-2" { "State" }
                        td ."border" ."p-2" { (trip.state.to_string()) }
                    }
                    tr {
                        td ."border" ."p-2" { "Location" }
                        td ."border" ."p-2" { (trip.location) }
                    }
                    tr {
                        td ."border" ."p-2" { "Start date" }
                        td ."border" ."p-2" { (trip.start_date) }
                    }
                    tr {
                        td ."border" ."p-2" { "End date" }
                        td ."border" ."p-2" { (trip.end_date) }
                    }
                    tr {
                        td ."border" ."p-2" { "Temp (min)" }
                        td ."border" ."p-2" { (trip.temp_min) }
                    }
                    tr {
                        td ."border" ."p-2" { "Temp (max)" }
                        td ."border" ."p-2" { (trip.temp_max) }
                    }
                    tr {
                        td ."border" ."p-2" { "Types" }
                        td ."border" ."p-2" {
                            ul
                                ."flex"
                                ."flex-row"
                                ."flex-wrap"
                                ."gap-2"
                                ."justify-between"
                            {
                                @let types = trip.types();
                                div
                                    ."flex"
                                    ."flex-row"
                                    ."flex-wrap"
                                    ."gap-2"
                                    ."justify-start"
                                {
                                    @for triptype in types.iter().filter(|t| t.active) {
                                        a href=(format!("type/{}/remove", triptype.id)) {
                                            li
                                                ."border"
                                                ."rounded-2xl"
                                                ."py-0.5"
                                                ."px-2"
                                                ."bg-green-100"
                                                ."cursor-pointer"
                                                ."flex"
                                                ."flex-column"
                                                ."items-center"
                                                ."hover:bg-red-200"
                                                ."gap-1"
                                            {
                                                span { (triptype.name) }
                                                span ."mdi" ."mdi-delete" ."text-sm" {}
                                            }
                                        }
                                    }
                                }
                                div
                                    ."flex"
                                    ."flex-row"
                                    ."flex-wrap"
                                    ."gap-2"
                                    ."justify-start"
                                {
                                    @for triptype in types.iter().filter(|t| !t.active) {
                                        a href=(format!("type/{}/add", triptype.id)) {
                                            li
                                                ."border"
                                                ."rounded-2xl"
                                                ."py-0.5"
                                                ."px-2"
                                                ."bg-gray-100"
                                                ."cursor-pointer"
                                                ."flex"
                                                ."flex-column"
                                                ."items-center"
                                                ."hover:bg-green-200"
                                                ."gap-1"
                                                ."opacity-60"
                                            {
                                                span { (triptype.name) }
                                                span ."mdi" ."mdi-plus" ."text-sm" {}
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

    pub fn into_markup(self) -> Markup {
        self.doc
    }
}
