import dominate
import dominate.tags as t
from dominate.util import raw

import urllib
import decimal

from .TripTable import TripTable
from .NewTrip import NewTrip
from .TripItemManager import TripItemManager

from ..helpers import *
from ..models import TripState


class TripInfoEditRow:
    def __init__(self, baseurl, name, value, attribute, inputtype="text"):

        with t.tr() as doc:
            t.form(
                id=f"edit-{attribute}",
                action=urllib.parse.urljoin(baseurl, f"edit/{attribute}/submit/"),
                target="_self",
                method="post",
                # data_hx_post=f"/list/{pkglist.id}/edit/submit",
                # data_hx_target="closest tr",
                # data_hx_swap="outerHTML",
            )
            with t.tr(_class=cls("h-full")):
                t.td(name, _class=cls("border", "p-2", "h-full"))
                with t.td(_class=cls("border", "p-0")):
                    t._input(
                        _class=cls("bg-blue-200", "w-full", "h-full", "px-2"),
                        type=inputtype,
                        id="item-edit-name",
                        form=f"edit-{attribute}",
                        name=attribute,
                        value=value,
                    )

                with t.td(
                    _class=cls(
                        "border",
                        "bg-red-200",
                        "hover:bg-red-400",
                        "cursor-pointer",
                        "w-8",
                        "text-center",
                    ),
                    id=f"edit-{attribute}-abort",
                ):
                    with t.a(href=baseurl):
                        with t.button():
                            t.span(_class=cls("mdi", "mdi-cancel", "text-xl")),
                with t.td(
                    id=f"edit-{attribute}-save",
                    _class=cls(
                        "border",
                        "bg-green-200",
                        "hover:bg-green-400",
                        "cursor-pointer",
                        "w-8",
                        "text-center",
                    ),
                ):
                    with t.button(type="submit", form=f"edit-{attribute}"):
                        t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
        self.doc = doc


class TripInfoNormalRow:
    def __init__(self, editable, baseurl, name, value, attribute):
        with t.tr() as doc:
            t.td(name, _class=cls("border", "p-2"))
            t.td(value, _class=cls("border", "p-2"))
            if editable:
                with t.td(
                    _class=cls(
                        "border",
                        "bg-blue-200",
                        "hover:bg-blue-400",
                        "cursor-pointer",
                        "w-8",
                        "text-center",
                    )
                ):
                    if editable:
                        with t.a(
                            # data_hx_post=f"/item/{item.id}/edit",
                            href=f"?edit={attribute}",
                            # data_hx_target="closest tr",
                            # data_hx_swap="outerHTML",
                        ):
                            with t.button():
                                t.span(_class=cls("mdi", "mdi-pencil", "text-xl")),

        self.doc = doc


class TripInfo:
    def __init__(self, trip, edit, baseurl, triptypes):
        with t.table(
            id="trip-table",
            _class=cls(
                "table",
                "table-auto",
                "border-collapse",
                "border-spacing-0",
                "border",
                "w-full",
            ),
        ) as doc:
            with t.tbody() as b:
                TripInfoNormalRow(False, "", "State", trip.state.name, "")

                if edit == "location":
                    TripInfoEditRow(baseurl, "Location", trip.location, "location")
                else:
                    TripInfoNormalRow(
                        True, baseurl, "Location", trip.location, "location"
                    )

                if edit == "start_date":
                    TripInfoEditRow(
                        baseurl,
                        "From",
                        str(trip.start_date),
                        "start_date",
                        inputtype="date",
                    )
                else:
                    TripInfoNormalRow(
                        True, baseurl, "From", str(trip.start_date), "start_date"
                    )

                if edit == "end_date":
                    TripInfoEditRow(
                        baseurl, "To", str(trip.end_date), "end_date", inputtype="date"
                    )
                else:
                    TripInfoNormalRow(
                        True, baseurl, "To", str(trip.end_date), "end_date"
                    )

                if edit == "temp_min":
                    TripInfoEditRow(
                        baseurl,
                        "Temp (min)",
                        trip.temp_min,
                        "temp_min",
                        inputtype="number",
                    )
                else:
                    TripInfoNormalRow(
                        True, baseurl, "Temp (min)", trip.temp_min, "temp_min"
                    )

                if edit == "temp_max":
                    TripInfoEditRow(
                        baseurl,
                        "Temp (max)",
                        trip.temp_max,
                        "temp_max",
                        inputtype="number",
                    )
                else:
                    TripInfoNormalRow(
                        True, baseurl, "Temp (max)", trip.temp_max, "temp_max"
                    )

                with t.tr():
                    t.td(f"Types", _class=cls("border", "p-2"))
                    with t.td(_class=cls("border", "p-2")):
                        with t.ul(
                            _class=cls(
                                "flex",
                                "flex-row",
                                "flex-wrap",
                                "gap-2",
                                "justify-between",
                            )
                        ):
                            with t.div(
                                _class=cls(
                                    "flex",
                                    "flex-row",
                                    "flex-wrap",
                                    "gap-2",
                                    "justify-start",
                                )
                            ):
                                for triptype in trip.types:
                                    with t.a(href=f"?type_remove={triptype.id}"):
                                        with t.li(
                                            _class=cls(
                                                "border",
                                                "rounded-2xl",
                                                "py-0.5",
                                                "px-2",
                                                "bg-green-100",
                                                "cursor-pointer",
                                                "flex",
                                                "flex-column",
                                                "items-center",
                                                "hover:bg-red-200",
                                                "gap-1",
                                            )
                                        ):
                                            t.span(triptype.name)
                                            t.span(
                                                _class=cls(
                                                    "mdi", "mdi-delete", "text-sm"
                                                )
                                            )

                            with t.div(
                                _class=cls(
                                    "flex",
                                    "flex-row",
                                    "flex-wrap",
                                    "gap-2",
                                    "justify-start",
                                )
                            ):
                                for triptype in triptypes:
                                    if triptype not in trip.types:
                                        with t.a(href=f"?type_add={triptype.id}"):
                                            with t.li(
                                                _class=cls(
                                                    "border",
                                                    "rounded-2xl",
                                                    "py-0.5",
                                                    "px-2",
                                                    "bg-gray-100",
                                                    "cursor-pointer",
                                                    "flex",
                                                    "flex-column",
                                                    "items-center",
                                                    "hover:bg-green-200",
                                                    "gap-1",
                                                    "opacity-60",
                                                )
                                            ):
                                                t.span(triptype.name)
                                                t.span(
                                                    _class=cls(
                                                        "mdi", "mdi-plus", "text-sm"
                                                    )
                                                )

                with t.tr():
                    t.td(f"Carried weight", _class=cls("border", "p-2"))
                    weight = sum(
                        [i.inventory_item.weight for i in trip.items if i.pick]
                    ) / decimal.Decimal(1000)
                    t.td(f"{weight} kg", _class=cls("border", "p-2"))

        self.doc = doc


class TripComments:
    def __init__(self, trip, baseurl):
        with t.div() as doc:
            t.h1("Comments", _class=cls("text-xl", "mb-5"))
            t.form(
                id="edit-comment",
                action=urllib.parse.urljoin(baseurl, f"edit/comment/submit/"),
                target="_self",
                method="post",
                # data_hx_post=f"/list/{pkglist.id}/edit/submit",
                # data_hx_target="closest tr",
                # data_hx_swap="outerHTML",
            )
            # https://stackoverflow.com/a/48460773
            t.textarea(
                trip.comment or "",
                name="comment",
                form="edit-comment",
                _class=cls("border", "w-full", "h-48"),
                oninput='this.style.height = "";this.style.height = this.scrollHeight + 2 + "px"',
            )

            with t.button(
                type="submit",
                form=f"edit-comment",
                _class=cls(
                    "mt-2",
                    "border",
                    "bg-green-200",
                    "hover:bg-green-400",
                    "cursor-pointer",
                    "flex",
                    "flex-column",
                    "p-2",
                    "gap-2",
                    "items-center",
                ),
            ):
                t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
                t.span("Save")

        self.doc = doc


class TripActions:
    def __init__(self, trip):
        with t.div() as doc:
            t.h1("Actions", _class=cls("text-xl", "mb-5"))

            with t.div(_class=cls("flex", "flex-column", "gap-2")):
                if trip.state == TripState.Planning:
                    t.button("Finish planning", _class=cls("border", "p-2"))
                if trip.state in (TripState.Planned, TripState.Active):
                    t.button("Start review", _class=cls("border", "p-2"))
                if trip.state == TripState.Done:
                    t.button("Back to review", _class=cls("border", "p-2"))

        self.doc = doc


class TripManager:
    def __init__(self, trip, categories, active_category, edit, baseurl, triptypes):
        with t.div(id="trips-manager", _class=cls("p-8")) as doc:
            if edit == "name":
                t.form(
                    id=f"edit-name",
                    action=urllib.parse.urljoin(baseurl, f"edit/name/submit/"),
                    target="_self",
                    method="post",
                    # data_hx_post=f"/list/{pkglist.id}/edit/submit",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                )
                with t.div(_class=cls("flex", "flex-row", "items-center", "gap-x-3")):
                    t._input(
                        _class=cls("bg-blue-200", "w-full", "h-full", "px-2", "text-2xl", "font-semibold"),
                        type="text",
                        id="item-edit-name",
                        form=f"edit-name",
                        name="name",
                        value=trip.name,
                    )
                    with t.a(href=baseurl):
                        with t.button(_class=cls(
                            "bg-red-200",
                            "hover:bg-red-400",
                            "cursor-pointer")):
                            t.span(_class=cls("mdi", "mdi-cancel", "text-xl")),
                    with t.button(type="submit", form=f"edit-name", _class=cls(
                            "bg-green-200",
                            "hover:bg-green-400",
                            "cursor-pointer")):
                        t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
            else:
                with t.div(_class=cls("flex", "flex-row", "items-center", "gap-x-3")):
                    t.h1(trip.name, _class=cls("text-2xl", "font-semibold"))
                    with t.span():
                        with t.a(
                            # data_hx_post=f"/item/{item.id}/edit",
                            href=f"?edit=name",
                            # data_hx_target="closest tr",
                            # data_hx_swap="outerHTML",
                        ):
                            with t.button():
                                t.span(_class=cls("mdi", "mdi-pencil", "text-xl", "opacity-50")),

            with t.div(_class=cls("my-6")):
                TripInfo(trip, edit, baseurl, triptypes).doc

            with t.div(_class=cls("my-6")):
                TripComments(trip, baseurl).doc

            with t.div(_class=cls("my-6")):
                TripActions(trip).doc

            with t.div(_class=cls("my-6")):
                TripItemManager(
                    trip, categories=categories, active_category=active_category
                ).doc

        self.doc = doc
