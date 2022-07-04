import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


def PackageListTable(pkglists):
    doc = t.div(id="packagelist-table")
    with doc:
        t.h1("Package Lists", _class=cls("text-2xl", "mb-5"))
        with t.table(
            id="packagelist-table",
            _class=cls(
                "table",
                "table-auto",
                # "border-separate",
                "border-collapse",
                "border-spacing-0",
                "border",
                "w-full",
            ),
        ):
            with t.thead(_class=cls("bg-gray-200")):
                t.tr(
                    t.th("Name", _class=cls("border", "p-2")),
                    t.th("Description", _class=cls("border", "p-2")),
                    t.th(_class=cls("border p-2")),
                    t.th(_class=cls("border p-2")),
                    t.th(_class=cls("border p-2")),
                    _class="h-10",
                )
            with t.tbody(data_hx_target="closest tr", data_hx_swap="outerHTML"):
                for pkglist in pkglists:
                    t.tr(
                        t.td(pkglist.name, _class=cls("border", "px-2")),
                        t.td(str(pkglist.description), _class=cls("border", "px-2")),
                        t.td(
                            t.span(_class=cls("mdi", "mdi-delete", "text-xl")),
                            id="delete-packagelist",
                            data_hx_delete=f"/list/{pkglist.id}",
                            _class=cls(
                                "border",
                                "bg-red-200",
                                "hover:bg-red-400",
                                "cursor-pointer",
                                "w-8",
                                "text-center",
                            ),
                        ),
                        t.td(
                            t.span(_class=cls("mdi", "mdi-pencil", "text-xl")),
                            id="edit-packagelist",
                            data_hx_post=f"/list/{pkglist.id}/edit",
                            _class=cls(
                                "border",
                                "bg-blue-200",
                                "hover:bg-blue-400",
                                "cursor-pointer",
                                "w-8",
                                "text-center",
                            ),
                        ),
                        t.td(
                            t.span(_class=cls("mdi", "mdi-arrow-right", "text-xl")),
                            id="edit-packagelist",
                            # data_hx_post=f"/list/{pkglist.id}/edit",
                            _class=cls(
                                "border",
                                "bg-green-200",
                                "hover:bg-green-400",
                                "cursor-pointer",
                                "w-8",
                                "text-center",
                            ),
                        ),
                        _class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-200"),
                    )

    return doc
