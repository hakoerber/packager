use crate::ClientState;
use maud::{html, Markup};

pub(crate) struct TypeList;

impl TypeList {
    #[tracing::instrument]
    pub fn build(state: &ClientState, trip_types: Vec<super::super::model::TripsType>) -> Markup {
        html!(
            div ."p-8" ."flex" ."flex-col" ."gap-8" {
                h1 ."text-2xl" {"Trip Types"}

                ul
                    ."flex"
                    ."flex-col"
                    ."items-stretch"
                    ."border-t"
                    ."border-l"
                    ."h-full"
                {
                    @for trip_type in trip_types {
                        li
                            ."border-b"
                            ."border-r"
                            ."flex"
                            ."flex-row"
                            ."justify-between"
                            ."items-stretch"
                        {
                            @if state.trip_type_edit == Some(trip_type.id) {
                                form
                                    ."hidden"
                                    id="edit-trip-type"
                                    action={ (trip_type.id) "/edit/name/submit" }
                                    target="_self"
                                    method="post"
                                {}
                                div
                                    ."bg-blue-200"
                                    ."p-2"
                                    ."grow"
                                {
                                    input
                                        ."bg-blue-100"
                                        ."hover:bg-white"
                                        ."w-full"
                                        type="text"
                                        name="new-value"
                                        form="edit-trip-type"
                                        value=(trip_type.name)
                                    {}
                                }
                                div
                                    ."flex"
                                    ."flex-row"
                                {
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
                                        form="edit-trip-type"
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
                            } @else {
                                span
                                    ."p-2"
                                {
                                    (trip_type.name)
                                }

                                div
                                    ."bg-blue-100"
                                    ."hover:bg-blue-200"
                                    ."p-0"
                                    ."w-8"
                                {
                                    a
                                        href={ "?edit=" (trip_type.id) }
                                        .flex
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
                    }
                }

                form
                    name="new-trip-type"
                    action="/trips/types/"
                    target="_self"
                    method="post"
                    ."mt-8" ."p-5" ."border-2" ."border-gray-200"
                {
                    div ."mb-5" ."flex" ."flex-row" {
                        span ."mdi" ."mdi-playlist-plus" ."text-2xl" ."mr-4" {}
                        p ."inline" ."text-xl" { "Add new trip type" }
                    }
                    div ."w-11/12" ."m-auto" {
                        div ."mx-auto" ."pb-8" {
                            div ."flex" ."flex-row" ."justify-center" {
                                label for="new-trip-type-name" ."font-bold" ."w-1/2" ."p-2" ."text-center" { "Name" }
                                span ."w-1/2" {
                                    input
                                        type="text"
                                        id="new-trip-type-name"
                                        name="new-trip-type-name"
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
            }
        )
    }
}
