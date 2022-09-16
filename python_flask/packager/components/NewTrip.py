import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


def NewTrip(name=None):
    with t.form(
        id="new-trip",
        name="new_trip",
        # data_hx_post="/list/",
        # data_hx_target="#trip-manager",
        # data_hx_swap="outerHTML",
        action="/trip/",
        target="_self",
        method="post",
        _class=cls("mt-8", "p-5", "border-2", "border-gray-200"),
        # **{
        #     "x-on:htmx:before-request": "(e) => submit_enabled || e.preventDefault()",
        #     "x-data": alpinedata(
        #         {
        #             "submit_enabled": (
        #                 jsbool(not error)
        #                 + '&& document.getElementById("listname").value.trim().length !== 0'
        #             )
        #         }
        #     ),
        # },
    ) as doc:
        with t.div(_class=cls("mb-5", "flex", "flex-row", "trips-center")):
            t.span(_class=cls("mdi", "mdi-playlist-plus", "text-2xl", "mr-4"))
            t.p("Add new trip", _class=cls("inline", "text-xl"))
        with t.div(_class=cls("w-11/12", "mx-auto")):
            with t.div(_class=cls("pb-8")):
                with t.div(
                    _class=cls("flex", "flex-row", "justify-center", "trips-start")
                ):
                    t.label(
                        "Name",
                        _for="trip-name",
                        _class=cls("font-bold", "w-1/2", "p-2", "text-center"),
                    )
                    with t.span(_class=cls("w-1/2")):
                        t._input(
                            type="text",
                            id="trip-name",
                            name="name",
                            **{"value": name} if name is not None else {},
                            # data_hx_target="#new-trip",
                            # data_hx_post="/trip/name/validate",
                            # data_hx_swap="outerHTML",
                            # data_hx_trigger="changed",
                            _class=cls(
                                "block",
                                "w-full",
                                "p-2",
                                "bg-gray-50",
                                # "appearance-none" if error else None,
                                "border-2",
                                # "border-red-400" if error else "border-gray-300",
                                "rounded",
                                "focus:outline-none",
                                "focus:bg-white",
                                # "focus:border-purple-500" if not error else None,
                            ),
                            # **{
                            #     "x-on:input": "submit_enabled = $event.srcElement.value.trim().length !== 0;"
                            # },
                        )
                        # t.p(
                        #     errormsg, _class=cls("mt-1", "text-red-400", "text-sm")
                        # ) if error else None
            with t.div(
                _class=cls("flex", "flex-row", "justify-center", "trips-center", "pb-8")
            ):
                t.label(
                    "Start date",
                    _for="start-date",
                    _class=cls("font-bold", "w-1/2", "text-center"),
                )
                with t.span(_class=cls("w-1/2")):
                    t._input(
                        type="date",
                        id="start-date",
                        name="start-date",
                        _class=cls(
                            "block",
                            "w-full",
                            "p-2",
                            "bg-gray-50",
                            "appearance-none",
                            "border-2",
                            "border-gray-300",
                            "rounded",
                            "focus:outline-none",
                            "focus:bg-white",
                            "focus:border-purple-500",
                        ),
                    )
            with t.div(
                _class=cls("flex", "flex-row", "justify-center", "trips-center", "pb-8")
            ):
                t.label(
                    "End date",
                    _for="end-date",
                    _class=cls("font-bold", "w-1/2", "text-center"),
                )
                with t.span(_class=cls("w-1/2")):
                    t._input(
                        type="date",
                        id="end-date",
                        name="end-date",
                        _class=cls(
                            "block",
                            "w-full",
                            "p-2",
                            "bg-gray-50",
                            "appearance-none",
                            "border-2",
                            "border-gray-300",
                            "rounded",
                            "focus:outline-none",
                            "focus:bg-white",
                            "focus:border-purple-500",
                        ),
                    )
            t._input(
                type="submit",
                value="Add",
                # **{
                #     "x-bind:class": 'submit_enabled ? "cursor-pointer" : "cursor-not-allowed opacity-50"'
                # },
                _class=cls(
                    "py-2",
                    "border-2",
                    "rounded",
                    "border-gray-300",
                    "mx-auto",
                    "w-full",
                    # "hover:border-purple-500" if not error else None,
                    # "hover:bg-purple-200" if not error else None,
                ),
            )
    return doc
