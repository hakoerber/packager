use crate::models;
use crate::models::*;

use maud::{html, Markup, PreEscaped};
use uuid::Uuid;

use serde_variant::to_variant_name;

use crate::ClientState;
pub struct TripManager;

impl TripManager {
    pub fn build(trips: Vec<models::Trip>) -> Markup {
        html!(
            div ."p-8" {
                (TripTable::build(trips))
                (NewTrip::build())
            }
        )
    }
}

pub enum InputType {
    Text,
    Number,
    Date,
}

impl From<InputType> for &'static str {
    fn from(value: InputType) -> &'static str {
        match value {
            InputType::Text => "text",
            InputType::Number => "number",
            InputType::Date => "date",
        }
    }
}

pub struct TripTable;

impl TripTable {
    pub fn build(trips: Vec<models::Trip>) -> Markup {
        html!(
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
                            (TripTableRow::build(trip.id, &trip.name))
                            (TripTableRow::build(trip.id, &trip.date_start))
                            (TripTableRow::build(trip.id, &trip.date_end))
                            (TripTableRow::build(trip.id, (trip.date_end - trip.date_start).whole_days()))
                            (TripTableRow::build(trip.id, trip.state))
                        }
                    }
                }
            }
        )
    }
}

pub struct TripTableRow;

impl TripTableRow {
    pub fn build(trip_id: Uuid, value: impl std::fmt::Display) -> Markup {
        html!(
            td ."border" ."p-0" ."m-0" {
                a ."inline-block" ."p-2" ."m-0" ."w-full"
                    href=(format!("/trip/{id}/", id=trip_id))
                { (value) }
            }
        )
    }
}

pub struct NewTrip;

impl NewTrip {
    pub fn build() -> Markup {
        html!(
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
        )
    }
}

pub struct Trip;

impl Trip {
    pub fn build(state: &ClientState, trip: &models::Trip) -> Markup {
        html!(
            div ."p-8" {
                div ."flex" ."flex-row" ."items-center" ."gap-x-3" {
                    h1 ."text-2xl" ."font-semibold"{ (trip.name) }
                }
                div ."my-6" {
                    (TripInfo::build(state, &trip))
                }
                div ."my-6" {
                    (TripComment::build(&trip))
                }
            }
        )
    }
}

pub struct TripInfoRow;

impl TripInfoRow {
    pub fn build(
        name: &str,
        value: impl std::fmt::Display,
        attribute_key: TripAttribute,
        edit_attribute: Option<&TripAttribute>,
        input_type: InputType,
    ) -> Markup {
        let edit = edit_attribute.map_or(false, |a| *a == attribute_key);
        html!(
            @if edit {
                form
                    name="edit-trip"
                    id="edit-trip"
                    action=(format!("edit/{key}/submit", key=(to_variant_name(&attribute_key).unwrap()) ))
                    // hx-post=(format!("edit/{name}/submit"))
                    target="."
                    method="post"
                ;
            }
            tr .h-full {
                @if edit {
                    td ."border" ."p-2" { (name) }
                    td ."border" ."bg-blue-300" ."px-2" ."py-0" {
                        div ."h-full" ."w-full" ."flex" {
                            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                                type=(<InputType as Into<&'static str>>::into(input_type))
                                id="new-value"
                                name="new-value"
                                form="edit-trip"
                                value=(value)
                            ;
                        }
                    }
                    td
                        ."border-none"
                        ."bg-red-100"
                        ."hover:bg-red-200"
                        ."p-0"
                        ."h-full"
                        ."w-8"
                    {
                        a
                            ."aspect-square"
                            ."flex"
                            ."w-full"
                            ."h-full"
                            ."p-0"
                            href="." // strips query parameters
                        {
                            span
                                ."m-auto"
                                ."mdi"
                                ."mdi-cancel"
                                ."text-xl";
                        }
                    }
                    td
                        ."border-none"
                        ."bg-green-100"
                        ."hover:bg-green-200"
                        ."p-0"
                        ."h-full"
                        ."w-8"
                    {
                        button
                            ."aspect-square"
                            ."flex"
                            ."w-full"
                            ."h-full"
                            type="submit"
                            form="edit-trip"
                        {
                            span
                                ."m-auto"
                                ."mdi"
                                ."mdi-content-save"
                                ."text-xl";
                        }
                    }
                } @else {
                    td ."border" ."p-2" { (name) }
                    td ."border" ."p-2" { (value) }
                    td
                        ."border-none"
                        ."bg-blue-100"
                        ."hover:bg-blue-200"
                        ."p-0"
                        ."w-8"
                        ."h-full"
                    {
                        a
                            .flex
                            ."w-full"
                            ."h-full"
                            href={ "?edit=" (to_variant_name(&attribute_key).unwrap()) }
                        {
                            span
                                ."m-auto"
                                ."mdi"
                                ."mdi-pencil"
                                ."text-xl";
                        }
                    }
                }
            }
        )
    }
}

pub struct TripInfo;

impl TripInfo {
    pub fn build(state: &ClientState, trip: &models::Trip) -> Markup {
        html!(
            table
                ."table"
                ."table-auto"
                ."border-collapse"
                ."border-spacing-0"
                ."border"
                ."w-full"
            {
                tbody {
                    (TripInfoRow::build("Location", &trip.location, TripAttribute::Location, state.trip_edit_attribute.as_ref(), InputType::Text))
                    (TripInfoRow::build("Start date", trip.date_start, TripAttribute::DateStart, state.trip_edit_attribute.as_ref(), InputType::Date))
                    (TripInfoRow::build("End date", trip.date_end, TripAttribute::DateEnd, state.trip_edit_attribute.as_ref(), InputType::Date))
                    (TripInfoRow::build("Temp (min)", trip.temp_min, TripAttribute::TempMin, state.trip_edit_attribute.as_ref(), InputType::Number))
                    (TripInfoRow::build("Temp (max)", trip.temp_max, TripAttribute::TempMax, state.trip_edit_attribute.as_ref(), InputType::Number))
                    tr .h-full {
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
        )
    }
}

pub struct TripComment;

impl TripComment {
    pub fn build(trip: &models::Trip) -> Markup {
        html!(
            h1 ."text-xl" ."mb-5" { "Comments" }

            form
                id="edit-comment"
                action="comment/submit"
                target="_self"
                method="post"
                ;

            // https://stackoverflow.com/a/48460773
            textarea
                #"comment"
                ."border" ."w-full" ."h-48"
                name="new-comment"
                form="edit-comment"
                autocomplete="off"
                oninput=r#"this.style.height = "";this.style.height = this.scrollHeight + 2 + "px""#
            { (trip.comment.as_ref().unwrap_or(&"".to_string())) }
            script defer { (PreEscaped(r#"e = document.getElementById("comment"); e.style.height = e.scrollHeight + 2 + "px";"#)) }

            button
                type="submit"
                form="edit-comment"
                ."mt-2"
                ."border"
                ."bg-green-200"
                ."hover:bg-green-400"
                ."cursor-pointer"
                ."flex"
                ."flex-column"
                ."p-2"
                ."gap-2"
                ."items-center"
            {
                span ."mdi" ."mdi-content-save" ."text-xl";
                span { "Save" }
            }
        )
    }
}
