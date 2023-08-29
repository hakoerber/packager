use maud::{html, Markup};

pub struct Home;

impl Home {
    pub fn build() -> Markup {
        html!(
            div
                id="home"
                hx-boost="true"
                ."p-8"
                ."flex"
                ."flex-col"
                ."gap-8"
                ."flex-nowrap"
            {
                h1
                    ."text-2xl"
                    ."m-auto"
                    ."my-4"
                {
                    "Welcome!"
                }
                section
                    ."border-2"
                    ."border-gray-200"
                    ."flex"
                    ."flex-row"
                {
                    a
                        href="/inventory/"
                        hx-boost="true"
                        ."p-8"
                        ."w-1/5"
                        ."flex"
                        ."hover:bg-gray-200"
                    {
                        span
                            ."m-auto"
                            ."text-xl"
                        { "Inventory" }
                    }
                    div
                        ."p-8"
                        ."w-4/5"
                        ."flex"
                        ."flex-col"
                        ."gap-3"
                    {
                        p {
                            "The inventory contains all the items that you own."
                        }
                        p {
                            "It is effectively a list of items, sectioned into
                            arbitrary categories"
                        }
                        p {
                            "Each item has some important data attached to it,
                            like its weight"
                        }
                    }
                }
                section
                    ."border-2"
                    ."border-gray-200"
                    ."flex"
                    ."flex-row"
                {
                    a
                        href="/trips/"
                        hx-boost="true"
                        ."p-8"
                        ."w-1/5"
                        ."flex"
                        ."hover:bg-gray-200"
                    {
                        span
                            ."m-auto"
                            ."text-xl"
                        { "Trips" }
                    }
                    div
                        ."p-8"
                        ."w-4/5"
                        ."flex"
                        ."flex-col"
                        ."gap-6"
                    {
                        div
                            ."flex"
                            ."flex-col"
                            ."gap-3"
                        {
                            p {
                                "Trips is where it gets interesting, as you can put
                                your inventory to good use"
                            }
                            p {
                                r#"With trips, you record any trips you plan to do. A
                                "trip" can be anything you want it to be. Anything
                                from a multi-week hike, a high altitude mountaineering
                                tour or just a visit to the library. Whenever it makes
                                sense to do some planning, creating a trip makes sense."#
                            }
                            p {
                                "Each trip has some metadata attached to it, like start-
                                and end dates or the expected temperature."
                            }
                        }
                        div
                            ."flex"
                            ."flex-col"
                            ."gap-3"
                        {
                            div
                                ."flex"
                                ."flex-row"
                                ."gap-2"
                                ."items-center"
                                ."justify-start"
                            {
                                span
                                    ."mdi"
                                    ."mdi-pound"
                                    ."text-lg"
                                    ."text-gray-300"
                                {}
                                h3 ."text-lg" {
                                    "States"
                                }
                            }
                            p {
                                "One of the most important parts of each trip is
                                its " em{"state"} ", which determines certain
                                actions on the trip and can have the following values:"
                            }
                            table
                                ."table"
                                ."table-auto"
                                ."border-collapse"
                            {
                                tr
                                    ."border-b-2"
                                    ."last:border-b-0"
                                {
                                    td ."py-2" ."pr-4" ."border-r-2" {
                                        "Init"
                                    }
                                    td ."py-2" ."w-full" ."pl-4" {
                                        "The new trip was just created"
                                    }
                                }
                                tr
                                    ."border-b-2"
                                    ."last:border-b-0"
                                {
                                    td ."py-2" ."pr-4" ."border-r-2" {
                                        "Planning"
                                    }
                                    td ."py-2" ."w-full" ."pl-4" {
                                        "Now, you actually start planning the trip.
                                        Setting the location, going through your
                                        items to decide what to take with you."
                                    }
                                }
                                tr
                                    ."border-b-2"
                                    ."last:border-b-0"
                                {
                                    td ."py-2" ."pr-4" ."border-r-2" {
                                        "Planned"
                                    }
                                    td ."py-2" ."w-full" ."pl-4" {
                                        "You are done with the planning. It's time
                                        to pack up your stuff and get going."
                                    }
                                }
                                tr
                                    ."border-b-2"
                                    ."last:border-b-0"
                                {
                                    td ."py-2" ."pr-4" ."border-r-2" {
                                        "Active"
                                    }
                                    td ."py-2" ."w-full" ."pl-4" {
                                        "The trip is finally underway!"
                                    }
                                }
                                tr
                                    ."border-b-2"
                                    ."last:border-b-0"
                                {
                                    td ."py-2" ."pr-4" ."border-r-2" {
                                        "Review"
                                    }
                                    td ."py-2" ."w-full" ."pl-4" {
                                        div
                                            ."flex"
                                            ."flex-col"
                                            ."gap-2"
                                        {
                                            p {
                                                "You returned from your trip. It may make
                                                sense to take a look back and see what
                                                went well and what went not so well."
                                            }
                                            p {
                                                "Anything you missed? Any items that you
                                                took with you that turned out to be useless?
                                                Record it and you will remember on your next
                                                trip"
                                            }
                                        }
                                    }
                                }
                                tr
                                    ."border-b-2"
                                    ."last:border-b-0"
                                {
                                    td ."py-2" ."pr-4" ."border-r-2" {
                                        "Done"
                                    }
                                    td ."py-2" ."w-full" ."pl-4" {
                                        "Your review is done and the trip can be laid to rest"
                                    }
                                }
                            }
                        }
                        div
                            ."flex"
                            ."flex-col"
                            ."gap-3"
                        {
                            div
                                ."flex"
                                ."flex-row"
                                ."gap-2"
                                ."items-center"
                                ."justify-start"
                            {
                                span
                                    ."mdi"
                                    ."mdi-pound"
                                    ."text-lg"
                                    ."text-gray-300"
                                {}
                                h3 ."text-lg" {
                                    "Items"
                                }
                            }
                            p {
                                "Of course, you can use items defined in your
                                inventory in your trips"
                            }
                            p {
                                "Generally, all items are available to you in
                                the same way as the inventory. For each item,
                                there are two specific states for the trip: An
                                item can be " em{"picked"} ", which means that
                                you plan to take it on the trip, and it can
                                be " em{"packed"} ", which means that you actually
                                packed it into your bag (and therefore, you cannot
                                forget it any more)"
                            }
                        }
                        div
                            ."flex"
                            ."flex-col"
                            ."gap-3"
                        {
                            div
                                ."flex"
                                ."flex-row"
                                ."gap-2"
                                ."items-center"
                                ."justify-start"
                            {
                                span
                                    ."mdi"
                                    ."mdi-pound"
                                    ."text-lg"
                                    ."text-gray-300"
                                {}
                                h3 ."text-lg" {
                                    "Types & Presets"
                                }
                            }
                            p {
                                "Often, you will take a certain set of items to
                                certain trips. Whenever you plan to sleep outdoors,
                                it makes sense to take your sleeping bag and mat
                                with you"
                            }
                            p {
                                "To reflect this, you can attach " em {"types"} " "
                                "to your trips. Types define arbitrary characteristics
                                about a trip and reference a certain set of items."
                            }
                            p {
                                "Here are some examples of types that might make sense:"
                            }
                            ul
                                ."list-disc"
                                ."list-inside"
                            {
                                li {
                                    r#""Biking": Make sure to pack your helmet and
                                    some repair tools"#
                                }
                                li {
                                    r#""Climbing": You certainly don't want to forget
                                    your climbing shoes"#
                                }
                                li {
                                    r#""Rainy": Pack a rain jacket and some waterproof
                                    shoes"#
                                }
                            }
                            p {
                                "Types are super flexible, it's up to you how to use
                                them"
                            }
                        }
                    }
                }

            }
        )
    }
}
