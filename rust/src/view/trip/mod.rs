use crate::htmx;
use crate::models;

use maud::{html, Markup, PreEscaped};
use uuid::Uuid;

use serde_variant::to_variant_name;

pub struct TripManager;

pub mod packagelist;
pub mod types;

impl TripManager {
    pub fn build(trips: Vec<models::trips::Trip>) -> Markup {
        html!(
            div
                ."p-8"
                ."flex"
                ."flex-col"
                ."gap-8"
            {
                h1 ."text-2xl" {"Trips"}
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

fn trip_state_icon(state: &models::trips::TripState) -> &'static str {
    match state {
        models::trips::TripState::Init => "mdi-magic-staff",
        models::trips::TripState::Planning => "mdi-text-box-outline",
        models::trips::TripState::Planned => "mdi-clock-outline",
        models::trips::TripState::Active => "mdi-play",
        models::trips::TripState::Review => "mdi-magnify",
        models::trips::TripState::Done => "mdi-check",
    }
}

pub struct TripTable;

impl TripTable {
    pub fn build(trips: Vec<models::trips::Trip>) -> Markup {
        html!(
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
                        tr ."h-10" ."even:bg-gray-100" ."hover:bg-gray-100" ."h-full" {
                            (TripTableRow::build(trip.id, &trip.name))
                            (TripTableRow::build(trip.id, trip.date_start.to_string()))
                            (TripTableRow::build(trip.id, trip.date_end.to_string()))
                            (TripTableRow::build(trip.id, (trip.date_end - trip.date_start).whole_days()))
                            (TripTableRow::build(trip.id, html!(
                                span .flex .flex-row .items-center {
                                    span ."mdi" .(trip_state_icon(&trip.state)) ."text-xl" ."mr-2" {}
                                    span { (trip.state) }
                                }
                            )))
                        }
                    }
                }
            }
        )
    }
}

pub struct TripTableRow;

impl TripTableRow {
    pub fn build(trip_id: Uuid, value: impl maud::Render) -> Markup {
        html!(
            td ."border" ."p-0" ."m-0" {
                a
                    href={"/trips/" (trip_id) "/"}
                    hx-boost="true"
                    ."inline-block"
                    ."p-2"
                    ."m-0"
                    ."w-full"

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
                action="/trips/"
                target="_self"
                method="post"
                hx-boost="true"
                ."p-5" ."border-2" ."border-gray-200"
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
    pub fn build(
        trip: &models::trips::Trip,
        trip_edit_attribute: Option<&models::trips::TripAttribute>,
        active_category: Option<&models::trips::TripCategory>,
    ) -> Markup {
        html!(
            div ."p-8" ."flex" ."flex-col" ."gap-8" {
                div
                    ."flex"
                    ."flex-row"
                    ."justify-between"
                {
                    div
                        ."flex"
                        ."flex-row"
                        ."items-stretch"
                        ."gap-x-5"
                    {
                        a
                            href="/trips/"
                            hx-boost="true"
                            ."text-sm"
                            ."text-gray-500"
                            ."flex"
                        {
                            div
                                ."m-auto"
                            {
                                span
                                    ."mdi"
                                    ."mdi-arrow-left"
                                {}
                                "back"
                            }
                        }
                        div ."flex" ."flex-row" ."items-center" ."gap-x-3" {
                            @if trip_edit_attribute.map_or(false, |a| *a == models::trips::TripAttribute::Name) {
                                form
                                    id="edit-trip"
                                    action=(format!("edit/{}/submit", to_variant_name(&models::trips::TripAttribute::Name).unwrap()))
                                    hx-boost="true"
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
                                        {}
                                        a
                                            href="."
                                            hx-boost="true"
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
                                            {}
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
                                            {}
                                        }
                                    }
                                }
                            } @else {
                                h1 ."text-2xl" { (trip.name) }
                                span {
                                    a
                                        href={"?edit=" (to_variant_name(&models::trips::TripAttribute::Name).unwrap())}
                                        hx-boost="true"
                                    {
                                        span
                                            ."mdi"
                                            ."mdi-pencil"
                                            ."text-xl"
                                            ."opacity-50"
                                        {}
                                    }
                                }
                            }
                        }
                    }
                    a
                        href={"/trips/" (trip.id) "/packagelist/"}
                        hx-boost="true"
                        ."p-2"
                        ."border-2"
                        ."border-gray-500"
                        ."bg-blue-200"
                        ."hover:bg-blue-200"
                    {
                        "Show Package List"
                    }
                }
                (TripInfo::build(trip_edit_attribute, trip))
                (TripComment::build(trip))
                (TripItems::build(active_category, trip))
            }
        )
    }
}

pub struct TripInfoRow;

impl TripInfoRow {
    pub fn build(
        name: &str,
        value: Option<impl std::fmt::Display>,
        attribute_key: &models::trips::TripAttribute,
        edit_attribute: Option<&models::trips::TripAttribute>,
        input_type: InputType,
    ) -> Markup {
        let edit = edit_attribute.map_or(false, |a| a == attribute_key);
        html!(
            @if edit {
                form
                    name="edit-trip"
                    id="edit-trip"
                    action=(format!("edit/{key}/submit", key=(to_variant_name(&attribute_key).unwrap()) ))
                    hx-boost="true"
                    target="_self"
                    method="post"
                {}
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
                            {}
                        }
                    }
                    td
                        ."border"
                        ."border-solid"
                        ."border-gray-300"
                        ."p-0"
                        ."h-full"
                    {
                        div
                            ."flex"
                            ."flex-row"
                            ."items-stretch"
                            ."h-full"
                        {
                            div
                                ."bg-red-100"
                                ."hover:bg-red-200"
                                ."w-8"
                                ."h-full"
                            {
                                a
                                    href="." // strips query parameters
                                    hx-boost="true"
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
                            div
                                ."bg-green-100"
                                ."hover:bg-green-200"
                                ."w-8"
                                ."h-full"
                            {
                                button
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
                                        ."text-xl"
                                    {}
                                }
                            }
                        }
                    }
                } @else {
                    td ."border" ."p-2" { (name) }
                    td ."border" ."p-2" { (value.map_or(String::new(), |v| v.to_string())) }
                    td
                        ."border-none"
                        ."bg-blue-100"
                        ."hover:bg-blue-200"
                        ."p-0"
                        ."w-8"
                        ."h-full"
                    {
                        a
                            href={ "?edit=" (to_variant_name(&attribute_key).unwrap()) }
                            hx-boost="true"
                            ."flex"
                            ."w-full"
                            ."h-full"
                        {
                            span
                                ."m-auto"
                                ."mdi"
                                ."mdi-pencil"
                                ."text-xl"
                            {}
                        }
                    }
                }
            }
        )
    }
}

pub struct TripInfoTotalWeightRow;

impl TripInfoTotalWeightRow {
    pub fn build(trip_id: Uuid, value: i64) -> Markup {
        html!(
            span
                hx-trigger={
                    (htmx::Event::TripItemEdited.to_str()) " from:body"
                }
                hx-get={"/trips/" (trip_id) "/total_weight"}
            {
                (value)
            }
        )
    }
}

pub struct TripInfoStateRow;

impl TripInfoStateRow {
    pub fn build(trip_state: &models::trips::TripState) -> Markup {
        let prev_state = trip_state.prev();
        let next_state = trip_state.next();
        html!(
            tr .h-full {
                td ."border" ."p-2" { "State" }
                td ."border" {
                    span .flex .flex-row .items-center .justify-start ."gap-2" {
                        span ."mdi" .(trip_state_icon(trip_state)) ."text-2xl" ."pl-2" {}
                        span ."pr-2" ."py-2" { (trip_state) }
                    }
                }

                td
                    ."border-none"
                    ."p-0"
                    ."w-8"
                    ."h-full"
                {
                    div
                        ."h-full"
                        ."flex"
                        ."flex-row"
                        ."items-stretch"
                        ."justify-stretch"
                    {
                        @if let Some(ref prev_state) = prev_state {
                            div
                                ."w-8"
                                ."grow"
                                ."h-full"
                                ."bg-yellow-100"
                                ."hover:bg-yellow-200"
                            {
                                form
                                    hx-post={"./state/" (prev_state)}
                                    hx-target="closest tr"
                                    hx-swap="outerHTML"
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
                                            ."text-xl"
                                        {}
                                    }
                                }
                            }
                        }
                        @if let Some(ref next_state) = next_state {
                            div
                                ."w-8"
                                ."grow"
                                ."h-full"
                                ."bg-green-100"
                                ."hover:bg-green-200"
                            {
                                form
                                    hx-post={"./state/" (next_state)}
                                    hx-target="closest tr"
                                    hx-swap="outerHTML"
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
                                            ."text-xl"
                                        {}
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

pub struct TripInfo;

impl TripInfo {
    pub fn build(
        trip_edit_attribute: Option<&models::trips::TripAttribute>,
        trip: &models::trips::Trip,
    ) -> Markup {
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
                        &models::trips::TripAttribute::Location,
                        trip_edit_attribute,
                        InputType::Text,
                    ))
                    (TripInfoRow::build("Start date",
                        Some(trip.date_start),
                        &models::trips::TripAttribute::DateStart,
                        trip_edit_attribute,
                        InputType::Date,
                    ))
                    (TripInfoRow::build("End date",
                        Some(trip.date_end),
                        &models::trips::TripAttribute::DateEnd,
                        trip_edit_attribute,
                        InputType::Date,
                    ))
                    (TripInfoRow::build("Temp (min)",
                        trip.temp_min,
                        &models::trips::TripAttribute::TempMin,
                        trip_edit_attribute,
                        InputType::Number,
                    ))
                    (TripInfoRow::build("Temp (max)",
                        trip.temp_max,
                        &models::trips::TripAttribute::TempMax,
                        trip_edit_attribute,
                        InputType::Number,
                    ))
                    (TripInfoStateRow::build(&trip.state))
                    tr .h-full {
                        td ."border" ."p-2" { "Types" }
                        td
                            colspan="2"
                            ."border"
                        {
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
                                    @let active_triptypes = types.iter().filter(|t| t.active).collect::<Vec<&models::trips::TripType>>();
                                    @let inactive_triptypes = types.iter().filter(|t| !t.active).collect::<Vec<&models::trips::TripType>>();

                                    @if !active_triptypes.is_empty() {
                                        div
                                            ."flex"
                                            ."flex-row"
                                            ."flex-wrap"
                                            ."gap-2"
                                            ."justify-start"
                                        {
                                            @for triptype in active_triptypes {
                                                a
                                                    href={"type/" (triptype.id) "/remove"}
                                                    hx-boost="true"
                                                {
                                                    li
                                                        ."border"
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
                                                a
                                                    href={"type/" (triptype.id) "/add"}
                                                    hx-boost="true"
                                                {
                                                    li
                                                        ."border"
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
                                a
                                    href="/trips/types/"
                                    hx-boost="true"
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
                        td
                            colspan="2"
                            ."border"
                            ."p-2"
                        {
                            (TripInfoTotalWeightRow::build(trip.id, trip.total_picked_weight()))
                        }
                    }
                }
            }
        )
    }
}

pub struct TripComment;

impl TripComment {
    pub fn build(trip: &models::trips::Trip) -> Markup {
        html!(
            div
                x-data="{ save_active: false }"
            {
                h1 ."text-xl" ."mb-5" { "Comments" }

                form
                    id="edit-comment"
                    action="comment/submit"
                    hx-boost="true"
                    target="_self"
                    method="post"
                {}

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
                    span ."mdi" ."mdi-content-save" ."text-xl" {}
                    span { "Save" }
                }
            }
        )
    }
}

pub struct TripItems;

impl TripItems {
    pub fn build(
        active_category: Option<&models::trips::TripCategory>,
        trip: &models::trips::Trip,
    ) -> Markup {
        html!(
            div #trip-items ."grid" ."grid-cols-4" ."gap-3" {
                div ."col-span-2" {
                    (TripCategoryList::build(active_category, trip))
                }
                div ."col-span-2" {
                    h1 ."text-2xl" ."mb-5" ."text-center" { "Items" }
                    @if let Some(active_category) = active_category {
                        (TripItemList::build(
                            trip.id,
                            active_category.items.as_ref().unwrap()
                            )
                        )
                    }
                }
            }
        )
    }
}

pub struct TripCategoryListRow;

impl TripCategoryListRow {
    pub fn build(
        trip_id: Uuid,
        category: &models::trips::TripCategory,
        active: bool,
        biggest_category_weight: i64,
        htmx_swap: bool,
    ) -> Markup {
        let has_new_items = category.items.as_ref().unwrap().iter().any(|item| item.new);
        html!(
            tr
                id={"category-" (category.category.id)}
                hx-swap-oob=[htmx_swap.then_some("outerHTML")]
                ."h-10"
                ."hover:bg-gray-100"
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
                            hx-post={
                                "/trips/" (trip_id)
                                "/categories/" (category.category.id)
                                "/select"
                            }
                            hx-target="#trip-items"
                            hx-swap="outerHTML"
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
                                {}
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
                        )
                    {}
                }
            }
        )
    }
}

pub struct TripCategoryList;

impl TripCategoryList {
    pub fn build(
        active_category: Option<&models::trips::TripCategory>,
        trip: &models::trips::Trip,
    ) -> Markup {
        let categories = trip.categories();

        let biggest_category_weight: i64 = categories
            .iter()
            .map(models::trips::TripCategory::total_picked_weight)
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
                        @let active = active_category.map_or(false, |c| category.category.id == c.category.id);
                        (TripCategoryListRow::build(trip.id, category, active, biggest_category_weight, false))
                    }
                    tr ."h-10" ."bg-gray-300" ."font-bold" {
                        td ."border" ."p-0" ."m-0" {
                            p ."p-2" ."m-2" { "Sum" }
                        }
                        td ."border" ."p-0" ."m-0" {
                            p ."p-2" ."m-2" {
                                (categories.iter().map(models::trips::TripCategory::total_picked_weight).sum::<i64>().to_string())
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
    pub fn build(trip_id: Uuid, items: &Vec<models::trips::TripItem>) -> Markup {
        let biggest_item_weight: i64 = items.iter().map(|item| item.item.weight).max().unwrap_or(1);

        html!(
            @if items.is_empty() {
                p ."text-lg" ."text-center" ."py-5" ."text-gray-400" { "[Empty]" }
            } @else {
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
                            th ."border" ."p-2" {}
                            th ."border" ."p-2" {}
                            th ."border" ."p-2" {}
                            th ."border" ."p-2" ."w-1/2" { "Name" }
                            th ."border" ."p-2" ."w-1/4" { "Weight" }
                        }
                    }
                    tbody {
                        @for item in items {
                            (TripItemListRow::build(trip_id, item, biggest_item_weight))
                        }
                    }
                }
            }
        )
    }
}

pub struct TripItemListRow;

impl TripItemListRow {
    pub fn build(
        trip_id: Uuid,
        item: &models::trips::TripItem,
        biggest_item_weight: i64,
    ) -> Markup {
        html!(
            tr ."h-10" {
                td
                    ."border"
                    ."p-0"
                {
                    a
                        href={
                            "/trips/" (trip_id)
                            "/items/" (item.item.id)
                            "/" (if item.picked { "unpick" } else { "pick" }) }
                        hx-post={
                            "/trips/" (trip_id)
                            "/items/" (item.item.id)
                            "/" (if item.picked { "unpick" } else { "pick" }) }
                        hx-target="closest tr"
                        hx-swap="outerHTML"
                        ."inline-block"
                        ."p-2"
                        ."m-0"
                        ."w-full"
                        ."justify-center"
                        ."content-center"
                        ."flex"
                        ."bg-green-200"[item.picked]
                        ."hover:bg-green-100"[!item.picked]
                    {
                        @if item.picked {
                            span
                                ."mdi"
                                ."mdi-clipboard-text-outline"
                                ."text-2xl"
                            {}
                        } @else {
                            span
                                ."mdi"
                                ."mdi-clipboard-text-off-outline"
                                ."text-2xl"
                            {}
                        }
                    }
                }
                td
                    ."border"
                    ."p-0"
                {
                    @if item.picked {
                        a
                            href={
                                "/trips/" (trip_id)
                                "/items/" (item.item.id)
                                "/" (if item.ready { "unready" } else { "ready" }) }
                            hx-post={
                                "/trips/" (trip_id)
                                "/items/" (item.item.id)
                                "/" (if item.ready { "unready" } else { "ready" }) }
                            hx-target="closest tr"
                            hx-swap="outerHTML"
                            ."inline-block"
                            ."p-2"
                            ."m-0"
                            ."w-full"
                            ."justify-center"
                            ."content-center"
                            ."flex"
                            ."bg-green-200"[item.ready]
                            ."hover:bg-green-100"[!item.ready]
                        {
                            @if item.ready {
                                span
                                    ."mdi"
                                    ."mdi-wardrobe-outline"
                                    ."text-2xl"
                                {}
                            } @else {
                                span
                                    ."mdi"
                                    ."mdi-map-marker-question-outline"
                                    ."text-2xl"
                                {}
                            }
                        }
                    } @else {
                        div
                            ."flex"
                            ."justify-center"
                            ."items-center"
                        {
                            span
                                ."mdi"
                                ."mdi-wardrobe-outline"
                                ."text-2xl"
                                ."text-gray-300"
                            {}
                        }
                    }
                }
                td
                    ."border"
                    ."p-0"
                {
                    @if item.picked {
                        a
                            href={
                                "/trips/" (trip_id)
                                "/items/" (item.item.id)
                                "/" (if item.packed { "unpack" } else { "pack" }) }
                            hx-post={
                                "/trips/" (trip_id)
                                "/items/" (item.item.id)
                                "/" (if item.packed { "unpack" } else { "pack" }) }
                            hx-target="closest tr"
                            hx-swap="outerHTML"
                            ."inline-block"
                            ."p-2"
                            ."m-0"
                            ."w-full"
                            ."justify-center"
                            ."content-center"
                            ."flex"
                            ."bg-green-200"[item.packed]
                            ."hover:bg-green-100"[!item.packed]
                        {
                            @if item.packed {
                                span
                                    ."mdi"
                                    ."mdi-bag-personal-outline"
                                    ."text-2xl"
                                {}
                            } @else {
                                span
                                    ."mdi"
                                    ."mdi-bag-personal-off-outline"
                                    ."text-2xl"
                                {}
                            }
                        }
                    } @else {
                        div
                            ."flex"
                            ."justify-center"
                            ."items-center"
                        {
                            span
                                ."mdi"
                                ."mdi-bag-personal-outline"
                                ."text-2xl"
                                ."text-gray-300"
                            {}
                        }
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
                            hx-boost="true"
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
                                {}
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
        )
    }
}
