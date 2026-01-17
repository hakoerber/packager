use crate::htmx;

use maud::{html, Markup, PreEscaped};
use uuid::Uuid;

use serde_variant::to_variant_name;

pub(crate) struct TripManager;

pub(crate) mod packagelist;
pub(crate) mod types;

use super::model;

use crate::domains::{self, view::View};

impl TripManager {
    #[tracing::instrument]
    pub fn build(trips: Vec<model::Trip>) -> Markup {
        html!(
            div
                ."p-8"
                ."flex"
                ."flex-col"
                ."gap-8"
            {
                h1 ."text-2xl" {"Trips"}
                (TripTable::build(&trips))
                (NewTrip::build(&trips))
            }
        )
    }
}

fn trip_state_icon(state: &model::TripState) -> &'static str {
    match state {
        model::TripState::Init => "mdi-magic-staff",
        model::TripState::Planning => "mdi-text-box-outline",
        model::TripState::Planned => "mdi-clock-outline",
        model::TripState::Active => "mdi-play",
        model::TripState::Review => "mdi-magnify",
        model::TripState::Done => "mdi-check",
    }
}

pub(crate) struct TripTable;

impl TripTable {
    #[tracing::instrument]
    pub fn build(trips: &Vec<model::Trip>) -> Markup {
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
                            (TripTableRow::build(trip.id, trip.date.start.to_string()))
                            (TripTableRow::build(trip.id, trip.date.end.to_string()))
                            (TripTableRow::build(trip.id, (trip.date.end - trip.date.start).whole_days()))
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

pub(crate) struct TripTableRow;

impl TripTableRow {
    #[tracing::instrument(skip(value))]
    pub fn build(trip_id: Uuid, value: impl maud::Render) -> Markup {
        html!(
            td ."border" ."p-0" ."m-0" {
                a
                    href={"/trips/" (trip_id) "/"}
                    ."inline-block"
                    ."p-2"
                    ."m-0"
                    ."w-full"

                { (value) }
            }
        )
    }
}

pub(crate) struct NewTrip;

impl NewTrip {
    #[tracing::instrument(skip(trips))]
    pub fn build(trips: &[model::Trip]) -> Markup {
        html!(
            form
                name="new_trip"
                action="/trips/"
                target="_self"
                method="post"
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
                            label for="start-date" ."font-bold" ."w-1/2" ."p-2" ."text-center" { "Start date" }
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
                            label for="end-date" ."font-bold" ."w-1/2" ."p-2" ."text-center" { "End date" }
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
                    div ."mx-auto" ."pb-8" {
                        div ."flex" ."flex-row" ."justify-center" {
                            label for="copy-from" ."font-bold" ."w-1/2" ."p-2" ."text-center" { "Reuse trip" }
                            span ."w-1/2" {
                                select
                                    id="copy-from"
                                    name="new-trip-copy-from"
                                    ."block"
                                    ."w-full"
                                    ."p-2"
                                    ."bg-gray-50"
                                    ."border-2"
                                    ."border-gray-300"
                                    ."focus:outline-none"
                                    ."focus:bg-white"
                                    ."focus:border-purple-500"
                                {
                                    option value="" { "[None]" }
                                    @for trip in trips.iter().rev() {
                                        option value=(trip.id) {
                                            (format!("{year}-{month:02} {name}",
                                                year = trip.date.start.year(),
                                                month = trip.date.start.month() as u8,
                                                name = trip.name
                                            ))
                                        }
                                    }
                                }
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

pub(crate) struct Trip;

impl Trip {
    #[tracing::instrument]
    pub fn build(
        trip: &model::Trip,
        trip_edit_attribute: Option<&model::TripAttribute>,
        active_category: Option<&model::TripCategory>,
        edit_todo: Option<Uuid>,
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
                            @if trip_edit_attribute.is_some_and(|a| *a == model::TripAttribute::Name) {
                                form
                                    id="edit-trip"
                                    action=(format!("edit/{}/submit", to_variant_name(&model::TripAttribute::Name).unwrap()))
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
                                            type="text"
                                            name="new-value"
                                            form="edit-trip"
                                            value=(trip.name)
                                        {}
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
                                        href={"?edit=" (to_variant_name(&model::TripAttribute::Name).unwrap())}
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
                (domains::trips::todos::List{todos: trip.todos(), trip}.build(domains::trips::todos::list::BuildInput { edit_todo}))
                (TripComment::build(trip))
                (TripItems::build(active_category, trip))
            }
        )
    }
}

pub(crate) trait Input {
    fn input(&self, id: &str, form: &str) -> Markup;
}

impl Input for Option<&String> {
    fn input(&self, id: &str, form: &str) -> Markup {
        html!(
            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                type="text"
                id=(id)
                name="new-value"
                form=(form)
                value=(self.unwrap_or(&String::new()))
            {}
        )
    }
}

impl Input for Option<&i32> {
    fn input(&self, id: &str, form: &str) -> Markup {
        html!(
            input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
                type="number"
                id=(id)
                name="new-value"
                form=(form)
                value=(self.map(|s| s.to_string()).unwrap_or_else(String::new))
            {}
        )
    }
}

impl Input for Option<&model::TripDate> {
    fn input(&self, _id: &str, _form: &str) -> Markup {
        html!(
            p { "this could be your form!" }
        )
    }
}

pub(crate) struct TripInfoRow<T>(std::marker::PhantomData<T>)
where
    T: std::fmt::Debug + std::fmt::Display;

impl<'a, T> Input for AttributeValue<'a, T>
where
    T: std::fmt::Debug + std::fmt::Display,
    Option<&'a T>: Input,
{
    fn input(&self, id: &str, form: &str) -> Markup {
        <Option<&'a T> as Input>::input(&self.0, id, form)
    }
}

#[derive(Debug)]
pub(crate) struct AttributeValue<'a, T>(pub Option<&'a T>)
where
    T: std::fmt::Debug + std::fmt::Display,
    Option<&'a T>: Input;

impl<'a, T> From<&'a Option<T>> for AttributeValue<'a, T>
where
    T: std::fmt::Debug + std::fmt::Display,
    Option<&'a T>: Input,
{
    fn from(value: &'a Option<T>) -> Self {
        Self(value.as_ref())
    }
}

impl<'a, T> From<&'a T> for AttributeValue<'a, T>
where
    T: std::fmt::Debug + std::fmt::Display,
    Option<&'a T>: Input,
{
    fn from(value: &'a T) -> Self {
        Self(Some(value))
    }
}

impl<'a, T> AttributeValue<'a, T>
where
    T: std::fmt::Debug + std::fmt::Display,
    Option<&'a T>: Input,
{
    fn input(&self, id: &str, form: &str) -> Markup {
        self.0.input(id, form)
    }
}
// html!(
//     input ."m-auto" ."px-1" ."block" ."w-full" ."bg-blue-100" ."hover:bg-white"
//         type="text"
//         id="new-value"
//         name="new-value"
//         form="edit-trip"
//         value=(self.0.map_or(String::new(), std::string::ToString::to_string))
//     {}
// )
//     }
// }

impl<T> TripInfoRow<T>
where
    T: std::fmt::Debug + std::fmt::Display,
{
    #[tracing::instrument]
    pub fn build<'a>(
        name: &str,
        value: AttributeValue<'a, T>,
        attribute_key: model::TripAttribute,
        edit_attribute: Option<&model::TripAttribute>,
    ) -> Markup
    where
        Option<&'a T>: Input,
    {
        let edit = edit_attribute.is_some_and(|a| *a == attribute_key);
        html!(
            @if edit {
                form
                    name="edit-trip"
                    id="edit-trip"
                    action=(format!("edit/{key}/submit", key=(to_variant_name(&attribute_key).unwrap()) ))
                    target="_self"
                    method="post"
                {}
            }
            tr .h-full {
                @if edit {
                    td ."border" ."p-2" { (name) }
                    td ."border" ."bg-blue-300" ."px-2" ."py-0" {
                        div ."h-full" ."w-full" ."flex" {
                            (value.input(attribute_key.id(), "edit-trip"))
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
                    td ."border" ."p-2" { (value.0.map_or(String::new(), std::string::ToString::to_string)) }
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

pub(crate) struct TripInfoTotalWeightRow;

impl TripInfoTotalWeightRow {
    #[tracing::instrument]
    pub fn build(trip_id: Uuid, value: i32) -> Markup {
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

pub(crate) struct TripInfoStateRow;

impl TripInfoStateRow {
    #[tracing::instrument]
    pub fn build(trip_state: &model::TripState) -> Markup {
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

pub(crate) struct TripInfo;

impl TripInfo {
    #[tracing::instrument]
    pub fn build(trip_edit_attribute: Option<&model::TripAttribute>, trip: &model::Trip) -> Markup {
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
                    @for row in model::view::info(trip, trip_edit_attribute) {
                        (row)
                    }

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
                                    @let active_triptypes = types.iter().filter(|t| t.active).collect::<Vec<&model::TripType>>();
                                    @let inactive_triptypes = types.iter().filter(|t| !t.active).collect::<Vec<&model::TripType>>();

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

pub(crate) struct TripComment;

impl TripComment {
    #[tracing::instrument]
    pub fn build(trip: &model::Trip) -> Markup {
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
                {}

                // https://stackoverflow.com/a/48460773
                textarea
                    # "comment"
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

pub(crate) struct TripItems;

impl TripItems {
    #[tracing::instrument]
    pub fn build(active_category: Option<&model::TripCategory>, trip: &model::Trip) -> Markup {
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

pub(crate) struct TripCategoryListRow;

impl TripCategoryListRow {
    #[tracing::instrument]
    pub fn build(
        trip_id: Uuid,
        category: &model::TripCategory,
        active: bool,
        biggest_category_weight: i32,
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
                                    f64::from(category.total_picked_weight())
                                    / f64::from(biggest_category_weight)
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

pub(crate) struct TripCategoryList;

impl TripCategoryList {
    #[tracing::instrument]
    pub fn build(active_category: Option<&model::TripCategory>, trip: &model::Trip) -> Markup {
        let categories = trip.categories();

        let biggest_category_weight: i32 = categories
            .iter()
            .map(model::TripCategory::total_picked_weight)
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
                        @let active = active_category.is_some_and(|c| category.category.id == c.category.id);
                        (TripCategoryListRow::build(trip.id, category, active, biggest_category_weight, false))
                    }
                    tr ."h-10" ."bg-gray-300" ."font-bold" {
                        td ."border" ."p-0" ."m-0" {
                            p ."p-2" ."m-2" { "Sum" }
                        }
                        td ."border" ."p-0" ."m-0" {
                            p ."p-2" ."m-2" {
                                (categories.iter().map(model::TripCategory::total_picked_weight).sum::<i32>().to_string())
                            }
                        }
                    }
                }
            }
        )
    }
}

pub(crate) struct TripItemList;

impl TripItemList {
    #[tracing::instrument]
    pub fn build(trip_id: Uuid, items: &Vec<model::TripItem>) -> Markup {
        let biggest_item_weight: i32 = items.iter().map(|item| item.item.weight).max().unwrap_or(1);

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

pub(crate) struct TripItemListRow;

impl TripItemListRow {
    #[tracing::instrument]
    pub fn build(trip_id: Uuid, item: &model::TripItem, biggest_item_weight: i32) -> Markup {
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
                    right:0;", width=(f64::from(item.item.weight) / f64::from(biggest_item_weight) * 100.0))) {}
                }
            }
        )
    }
}
