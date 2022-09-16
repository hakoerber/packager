import datetime

import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


class TripRow(object):
    def __init__(self, trip):
        with t.tr(
            _class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-100", "h-full")
        ) as doc:
            with t.td(_class=cls("border", "p-0", "m-0")):
                t.a(
                    trip.name,
                    id="select-trip",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"/trip/{trip.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                )
            with t.td(_class=cls("border", "p-0", "m-0")):
                t.a(
                    t.p(str(trip.start_date)),
                    id="select-trip",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"/trip/{trip.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                )
            with t.td(_class=cls("border", "p-0", "m-0")):
                t.a(
                    t.p(str(trip.end_date)),
                    id="select-trip",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"/trip/{trip.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                )
            with t.td(_class=cls("border", "p-0", "m-0")):
                t.a(
                    t.p((trip.end_date - trip.start_date).days),
                    id="select-trip",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"/trip/{trip.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                )
            today = datetime.datetime.now().date()
            with t.td(_class=cls("border", "p-0", "m-0")):
                t.a(
                    t.p(trip.state.name),
                    id="select-trip",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"/trip/{trip.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                )
        self.doc = doc


def TripTable(trips):
    with t.div() as doc:
        t.h1("Trips", _class=cls("text-2xl", "mb-5"))
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
        ):
            with t.thead(_class=cls("bg-gray-200")):
                t.tr(
                    t.th("Name", _class=cls("border", "p-2")),
                    t.th("From", _class=cls("border", "p-2")),
                    t.th("To", _class=cls("border", "p-2")),
                    t.th("Nights", _class=cls("border", "p-2")),
                    t.th("State", _class=cls("border", "p-2")),
                    _class="h-10",
                )
            with t.tbody() as b:
                for trip in trips:
                    TripRow(trip).doc

    return doc
