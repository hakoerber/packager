import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


class PackageListTableRowEdit:
    def __init__(self, pkglist):
        error, errormsg = pkglist.error, pkglist.errormsg
        assert not (error and not errormsg)
        with t.tr(
            _class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-200"),
            id="pkglist-edit-row",
            **{
                "x-data": '{ edit_submit_enabled: document.getElementById("listedit-name").value.trim().length !== 0 }'
            },
        ) as doc:
            with t.td(colspan=3, _class=cls("border-none", "bg-purple-100", "h-10")):
                with t.div():
                    t.form(
                        id="edit-pkglist",
                        action=f"/list/{pkglist.id}/edit/submit/",
                        target="_self",
                        method="post",
                        data_hx_post=f"/list/{pkglist.id}/edit/submit",
                        data_hx_target="closest tr",
                        data_hx_swap="outerHTML",
                    )
                if error:
                    t.p(errormsg, _class=cls("text-red-400", "text-sm"))
                with t.div(_class=cls("flex", "flex-row", "h-full")):
                    with t.div(
                        _class=cls(
                            "border",
                            "border-1",
                            "border-purple-500",
                            "bg-purple-100",
                            "mr-1",
                        )
                    ):
                        t._input(
                            _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                            type="text",
                            id="listedit-name",
                            form="edit-pkglist",
                            name="name",
                            value=pkglist.name if error else pkglist.name,
                            **{
                                "x-on:input": "edit_submit_enabled = $event.srcElement.value.trim().length !== 0;"
                            },
                        )
                    with t.div(
                        _class=cls(
                            "border", "border-1", "border-purple-500", "bg-purple-100"
                        )
                    ):
                        t._input(
                            _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                            type="text",
                            name="description",
                            form="edit-pkglist",
                            value=pkglist.description,
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
                id="edit-packagelist-abort",
            ):
                with t.a(
                    href="/",
                    data_hx_post=f"/list/{pkglist.id}/edit/cancel",
                    data_hx_target="closest tr",
                    data_hx_swap="outerHTML",
                ):
                    t.span(_class=cls("mdi", "mdi-cancel", "text-xl")),
            with t.td(
                id="edit-packagelist-save",
                _class=cls(
                    "border",
                    "bg-green-200",
                    "hover:bg-green-400",
                    "cursor-pointer",
                    "w-8",
                    "text-center",
                ),
                **{
                    "x-bind:class": 'edit_submit_enabled || "cursor-not-allowed opacity-50"',
                    "x-on:htmx:before-request": "(e) => edit_submit_enabled || e.preventDefault()",
                },
            ):
                with t.button(type="submit", form="edit-pkglist"):
                    t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
        self.doc = doc


class PackageListTableRowNormal:
    def __init__(self, pkglist):
        with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-200")) as doc:
            t.td(pkglist.name, _class=cls("border", "px-2")),
            t.td(str(pkglist.description), _class=cls("border", "px-2")),
            t.td(
                t.span(_class=cls("mdi", "mdi-delete", "text-xl")),
                id="delete-packagelist",
                data_hx_delete=f"/list/{pkglist.id}",
                data_hx_target="closest tr",
                data_hx_swap="outerHTML",
                _class=cls(
                    "border",
                    "bg-red-200",
                    "hover:bg-red-400",
                    "cursor-pointer",
                    "w-8",
                    "text-center",
                ),
            ),
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
                with t.a(
                    id="edit-packagelist",
                    data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"/?edit={pkglist.id}",
                    data_hx_target="closest tr",
                    data_hx_swap="outerHTML",
                ):
                    t.span(_class=cls("mdi", "mdi-pencil", "text-xl")),
            t.td(
                t.span(_class=cls("mdi", "mdi-arrow-right", "text-xl")),
                id="edit-packagelist",
                # data_hx_post=f"/list/{pkglist.id}/show",
                _class=cls(
                    "border",
                    "bg-green-200",
                    "hover:bg-green-400",
                    "cursor-pointer",
                    "w-8",
                    "text-center",
                ),
            ),
        self.doc = doc


class PackageListTableRow:
    def __init__(self, pkglist):
        if pkglist.edit:
            self.doc = PackageListTableRowEdit(pkglist).doc
        else:
            self.doc = PackageListTableRowNormal(pkglist).doc


def PackageListTable(pkglists):
    with t.div(id="packagelist-table") as doc:
        t.h1("Package Lists", _class=cls("text-2xl", "mb-5"))
        with t.table(
            id="packagelist-table",
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
                    t.th("Description", _class=cls("border", "p-2")),
                    t.th(_class=cls("border p-2")),
                    t.th(_class=cls("border p-2")),
                    t.th(_class=cls("border p-2")),
                    _class="h-10",
                )
            with t.tbody() as b:
                for pkglist in pkglists:
                    PackageListTableRow(pkglist).doc

    return doc
