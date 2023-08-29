use crate::models;
use crate::models::*;

use maud::{html, Markup, PreEscaped};
use uuid::Uuid;

use serde_variant::to_variant_name;

use crate::ClientState;
pub struct TripManager;

pub mod types;
pub use types::*;

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
                            (TripTableRow::build(trip.id, trip.date_start))
                            (TripTableRow::build(trip.id, trip.date_end))
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
    pub fn build(state: &ClientState, trip: &models::Trip) -> Result<Markup, Error> {
        Ok(html!(
            div ."p-8" ."flex" ."flex-col" ."gap-8" {
                div ."flex" ."flex-row" ."items-center" ."gap-x-3" {
                    @if state.trip_edit_attribute.as_ref().map_or(false, |a| *a == TripAttribute::Name) {
                        form
                            id="edit-trip"
                            action=(format!("edit/{}/submit", to_variant_name(&TripAttribute::Name).unwrap()))
                            target="_self"
                            method="post"
                        {
                            div
                                ."flex"
                                ."flex-row"
                                ."items-center"
                                ."gap-x-3"
                                ."items-stretch"
                            {
                                input
                                    ."bg-blue-200"
                                    ."w-full"
                                    ."text-2xl"
                                    ."font-semibold"
                                    type=(<InputType as Into<&'static str>>::into(InputType::Text))
                                    name="new-value"
                                    form="edit-trip"
                                    value=(trip.name)
                                ;
                                a
                                    href="."
                                    ."bg-red-200"
                                    ."hover:bg-red-300"
                                    ."w-8"
                                    ."flex"
                                {
                                    span
                                        ."mdi"
                                        ."mdi-cancel"
                                        ."text-xl"
                                        ."m-auto"
                                    ;
                                }
                                button
                                    type="submit"
                                    form="edit-trip"
                                    ."bg-green-200"
                                    ."hover:bg-green-300"
                                    ."w-8"
                                {
                                    span
                                        ."mdi"
                                        ."mdi-content-save"
                                        ."text-xl"
                                    ;
                                }
                            }
                        }
                    } @else {
                        h1 ."text-2xl" ."font-semibold"{ (trip.name) }
                        span {
                            a href=(format!("?edit={}", to_variant_name(&TripAttribute::Name).unwrap()))
                            {
                                span
                                    ."mdi"
                                    ."mdi-pencil"
                                    ."text-xl"
                                    ."opacity-50"
                                ;
                            }
                        }
                    }
                }
                (TripInfo::build(state, trip))
                (TripComment::build(trip))
                (TripItems::build(state, trip)?)
            }
        ))
    }
}

pub struct TripInfoRow;

impl TripInfoRow {
    pub fn build(
        name: &str,
        value: Option<impl std::fmt::Display>,
        attribute_key: &TripAttribute,
        edit_attribute: Option<&TripAttribute>,
        input_type: InputType,
        has_two_columns: bool,
    ) -> Markup {
        let edit = edit_attribute.map_or(false, |a| a == attribute_key);
        html!(
            @if edit {
                form
                    name="edit-trip"
                    id="edit-trip"
                    action=(format!("edit/{key}/submit", key=(to_variant_name(&attribute_key).unwrap()) ))
                    htmx-push-url="true"
                    target="_self"
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
                                value=(value.map_or(String::new(), |v| v.to_string()))
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
                    td ."border" ."p-2" { (value.map_or(String::new(), |v| v.to_string())) }
                    td
                        colspan=(if has_two_columns {"2"} else {"1"})
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
        let has_two_columns =
            state.trip_edit_attribute.is_some() || !(trip.state.is_first() || trip.state.is_last());
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
                    (TripInfoRow::build("Location",
                        trip.location.as_ref(),
                        &TripAttribute::Location,
                        state.trip_edit_attribute.as_ref(),
                        InputType::Text,
                        has_two_columns
                    ))
                    (TripInfoRow::build("Start date",
                        Some(trip.date_start),
                        &TripAttribute::DateStart,
                        state.trip_edit_attribute.as_ref(),
                        InputType::Date,
                        has_two_columns
                    ))
                    (TripInfoRow::build("End date",
                        Some(trip.date_end),
                        &TripAttribute::DateEnd,
                        state.trip_edit_attribute.as_ref(),
                        InputType::Date,
                        has_two_columns
                    ))
                    (TripInfoRow::build("Temp (min)",
                        trip.temp_min,
                        &TripAttribute::TempMin,
                        state.trip_edit_attribute.as_ref(),
                        InputType::Number,
                        has_two_columns
                    ))
                    (TripInfoRow::build("Temp (max)",
                        trip.temp_max,
                        &TripAttribute::TempMax,
                        state.trip_edit_attribute.as_ref(),
                        InputType::Number,
                        has_two_columns
                    ))
                    tr .h-full {
                        td ."border" ."p-2" { "State" }
                        td ."border" ."p-2" { (trip.state) }
                        @let prev_state = trip.state.prev();
                        @let next_state = trip.state.next();

                        @if let Some(ref prev_state) = prev_state {
                            td
                                colspan=(if next_state.is_none() && has_two_columns { "2" } else { "1" })
                                ."border-none"
                                ."bg-yellow-100"
                                ."hover:bg-yellow-200"
                                ."p-0"
                                ."w-8"
                                ."h-full"
                            {
                                form
                                    action={"./state/" (prev_state)}
                                    method="post"
                                    ."flex"
                                    ."w-full"
                                    ."h-full"

                                {
                                    button
                                        type="submit"
                                        ."w-full"
                                        ."h-full"
                                    {
                                        span
                                            ."m-auto"
                                            ."mdi"
                                            ."mdi-step-backward"
                                            ."text-xl";
                                    }
                                }
                            }
                        }
                        @if let Some(ref next_state) = trip.state.next() {
                            td
                                colspan=(if prev_state.is_none() && has_two_columns { "2" } else { "1" })
                                ."border-none"
                                ."bg-green-100"
                                ."hover:bg-green-200"
                                ."p-0"
                                ."w-8"
                                ."h-full"
                            {
                                form
                                    action={"./state/" (next_state)}
                                    method="post"
                                    ."flex"
                                    ."w-full"
                                    ."h-full"

                                {
                                    button
                                        type="submit"
                                        ."w-full"
                                        ."h-full"
                                    {
                                        span
                                            ."m-auto"
                                            ."mdi"
                                            ."mdi-step-forward"
                                            ."text-xl";
                                    }
                                }
                            }
                        }
                    }
                    tr .h-full {
                        td ."border" ."p-2" { "Types" }
                        td ."border" {
                            div
                                ."flex"
                                ."flex-row"
                                ."items-center"
                                ."justify-between"
                            {
                                ul
                                    ."flex"
                                    ."flex-row"
                                    ."flex-wrap"
                                    ."gap-2"
                                    ."justify-between"
                                    ."p-2"
                                // as we have a gap between the elements, we have
                                // to completely skip an element when there are no
                                // active or inactive items, otherwise we get the gap
                                // between the empty (invisible) item, throwing off
                                // the margins
                                {
                                    @let types = trip.types();
                                    @let active_triptypes = types.iter().filter(|t| t.active).collect::<Vec<&TripType>>();
                                    @let inactive_triptypes = types.iter().filter(|t| !t.active).collect::<Vec<&TripType>>();

                                    @if !active_triptypes.is_empty() {
                                        div
                                            ."flex"
                                            ."flex-row"
                                            ."flex-wrap"
                                            ."gap-2"
                                            ."justify-start"
                                        {
                                            @for triptype in active_triptypes {
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
                                                        span ."mdi" ."mdi-close" ."text-sm" {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    @if !inactive_triptypes.is_empty() {
                                        div
                                            ."flex"
                                            ."flex-row"
                                            ."flex-wrap"
                                            ."gap-2"
                                            ."justify-start"
                                        {
                                            @for triptype in inactive_triptypes {
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
                                                        ."hover:opacity-100"
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
                                a href="/trips/types/"
                                    ."text-sm"
                                    ."text-gray-500"
                                    ."mr-2"
                                {
                                    "Manage" br; "types"
                                }
                            }
                        }
                    }
                    tr .h-full {
                        td ."border" ."p-2" { "Carried weight" }
                        td ."border" ."p-2"
                        {
                            (trip.total_picked_weight())
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
            div
                x-data="{ save_active: false }"
            {
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
                    x-on:input="save_active=true"
                    ."border" ."w-full" ."h-48"
                    name="new-comment"
                    form="edit-comment"
                    autocomplete="off"
                    oninput=r#"this.style.height = "";this.style.height = this.scrollHeight + 2 + "px""#
                { (trip.comment.as_ref().unwrap_or(&String::new())) }
                script defer { (PreEscaped(r#"e = document.getElementById("comment"); e.style.height = e.scrollHeight + 2 + "px";"#)) }

                button
                    type="submit"
                    form="edit-comment"
                    x-bind:disabled="!save_active"
                    ."enabled:bg-green-200"
                    ."enabled:hover:bg-green-400"
                    ."enabled:cursor-pointer"
                    ."disabled:opacity-50"
                    ."disabled:bg-gray-300"
                    ."mt-2"
                    ."border"
                    ."flex"
                    ."flex-column"
                    ."p-2"
                    ."gap-2"
                    ."items-center"
                {
                    span ."mdi" ."mdi-content-save" ."text-xl";
                    span { "Save" }
                }
            }
        )
    }
}

pub struct TripItems;

impl TripItems {
    pub fn build(state: &ClientState, trip: &models::Trip) -> Result<Markup, Error> {
        Ok(html!(
            div ."grid" ."grid-cols-4" ."gap-3" {
                div ."col-span-2" {
                    (TripCategoryList::build(state, trip))
                }
                div ."col-span-2" {
                    h1 ."text-2xl" ."mb-5" ."text-center" { "Items" }
                    @if let Some(active_category_id) = state.active_category_id {
                        (TripItemList::build(
                            state,
                            trip,
                            trip
                                .categories()
                                .iter()
                                .find(|category|
                                    category.category.id == active_category_id
                                )
                                .ok_or(
                                    Error::NotFound{
                                        description: format!("no category with id {active_category_id}")
                                    }
                                )?
                                .items
                                .as_ref()
                                .unwrap()
                            )
                        )
                    }
                }
            }
        ))
    }
}

pub struct TripCategoryList;

impl TripCategoryList {
    pub fn build(state: &ClientState, trip: &models::Trip) -> Markup {
        let categories = trip.categories();

        let biggest_category_weight: i64 = categories
            .iter()
            .map(TripCategory::total_picked_weight)
            .max()
            .unwrap_or(1);

        html!(
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
                        th ."border" ."p-2" ."w-2/5" { "Name" }
                        th ."border" ."p-2" { "Weight" }
                    }
                }
                tbody {
                    @for category in trip.categories() {
                        @let has_new_items = category.items.as_ref().unwrap().iter().any(|item| item.new);
                        @let active = state.active_category_id.map_or(false, |id| category.category.id == id);
                        tr
                            ."h-10"
                            ."hover:bg-purple-100"
                            ."m-3"
                            ."h-full"
                            ."outline"[active]
                            ."outline-2"[active]
                            ."outline-indigo-300"[active]
                        {

                            td

                                ."border"
                                ."m-0"

                            {
                                div
                                    ."p-0"
                                    ."flex"
                                    ."flex-row"
                                    ."items-center"
                                    ."group"
                                {
                                    a
                                        id="select-category"
                                        href=(
                                            format!(
                                                "?category={id}",
                                                id=category.category.id
                                            )
                                        )
                                        ."inline-block"
                                        ."p-2"
                                        ."m-0"
                                        ."w-full"
                                        ."grow"
                                        ."font-bold"[active]
                                    {
                                        (category.category.name.clone())
                                    }
                                    @if has_new_items {
                                        div
                                            ."mr-2"
                                            ."flex"
                                            ."flex-row"
                                            ."items-center"
                                        {
                                            p
                                                ."hidden"
                                                ."group-hover:inline"
                                                ."text-sm"
                                                ."text-gray-500"
                                                ."grow"
                                            {
                                                "new items"
                                            }
                                            span
                                                ."mdi"
                                                ."mdi-exclamation-thick"
                                                ."text-xl"
                                                ."text-yellow-400"
                                                ."grow-0"
                                            ;
                                        }
                                    }
                                }
                            }
                            td ."border" ."m-0" ."p-2" style="position:relative;" {
                                p {
                                    (category.total_picked_weight().to_string())
                                }
                                div ."bg-blue-600" ."h-1.5"
                                    style=(
                                        format!(
                                            "width: {width}%;position:absolute;left:0;bottom:0;right:0;",
                                            width=(
                                                (category.total_picked_weight() as f64)
                                                / (biggest_category_weight as f64)
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
                                (categories.iter().map(TripCategory::total_picked_weight).sum::<i64>().to_string())
                            }
                        }
                    }
                }
            }
        )
    }
}

pub struct TripItemList;

impl TripItemList {
    pub fn build(state: &ClientState, trip: &models::Trip, items: &Vec<TripItem>) -> Markup {
        let biggest_item_weight: i64 = items.iter().map(|item| item.item.weight).max().unwrap_or(1);

        html!(
            @if items.is_empty() {
                p ."text-lg" ."text-center" ."py-5" ."text-gray-400" { "[Empty]" }
            } @else {
                @if let Some(edit_item) = state.edit_item {
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
                    .table-fixed
                    ."border-collapse"
                    ."border-spacing-0"
                    ."border"
                    ."w-full"
                {
                    thead ."bg-gray-200" {
                        tr ."h-10" {
                            th ."border" ."p-2" { "Take?" }
                            th ."border" ."p-2" { "Packed?" }
                            th ."border" ."p-2" ."w-1/2" { "Name" }
                            th ."border" ."p-2" ."w-1/4" { "Weight" }
                        }
                    }
                    tbody {
                        @for item in items {
                            tr ."h-10" ."even:bg-gray-100" ."hover:bg-purple-100" {
                                td {
                                    a
                                        href={
                                            "/trip/" (trip.id)
                                            "/items/" (item.item.id)
                                            "/" (if item.picked { "unpick" } else { "pick" }) }
                                        ."inline-block"
                                        ."p-2"
                                        ."m-0"
                                        ."w-full"
                                        ."justify-center"
                                        ."content-center"
                                        ."flex"
                                    {
                                        input
                                            type="checkbox"
                                            checked[item.picked]
                                            autocomplete="off"
                                        ;
                                    }
                                }
                                td {
                                    a
                                        href={
                                            "/trip/" (trip.id)
                                            "/items/" (item.item.id)
                                            "/" (if item.packed { "unpack" } else { "pack" }) }
                                        ."inline-block"
                                        ."p-2"
                                        ."m-0"
                                        ."w-full"
                                        ."justify-center"
                                        ."content-center"
                                        ."flex"
                                    {
                                        input
                                            type="checkbox"
                                            checked[item.packed]
                                            autocomplete="off"
                                        ;
                                    }
                                }
                                td ."border" ."p-0" {
                                    div
                                        ."flex"
                                        ."flex-row"
                                        ."items-center"
                                    {
                                        a
                                            ."p-2" ."w-full" ."inline-block"
                                            href=(
                                                format!("/inventory/item/{id}/", id=item.item.id)
                                            )
                                        {
                                            (item.item.name.clone())
                                        }
                                        @if item.new {
                                            div ."mr-2" {
                                                span
                                                    ."mdi"
                                                    ."mdi-exclamation-thick"
                                                    ."text-xl"
                                                    ."text-yellow-400"
                                                    ."grow-0"
                                                ;
                                            }
                                        }
                                    }
                                }
                                td ."border" ."p-2" style="position:relative;" {
                                    p { (item.item.weight.to_string()) }
                                    div ."bg-blue-600" ."h-1.5" style=(format!("
                                    width: {width}%;
                                    position:absolute;
                                    left:0;
                                    bottom:0;
                                    right:0;", width=((item.item.weight as f64) / (biggest_item_weight as f64) * 100.0))) {}
                                }
                            }
                        }
                    }
                }
            }
        )
    }
}
