import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *

import urllib

class InventoryItemInfoEditRow:
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


class InventoryItemInfoNormalRow:
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


class InventoryItemInfo:
    def __init__(self, item, edit, baseurl):
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
                if edit == "name":
                    InventoryItemInfoEditRow(baseurl, "Name", item.name, "name")
                else:
                    InventoryItemInfoNormalRow(
                        True, baseurl, "Name", item.name, "name"
                    )

                if edit == "weight":
                    InventoryItemInfoEditRow(
                        baseurl,
                        "Weight",
                        str(item.weight),
                        "weight",
                        inputtype="number",
                    )
                else:
                    InventoryItemInfoNormalRow(
                        True, baseurl, "Weight", str(item.weight), "weight"
                    )

        self.doc = doc



class InventoryItemDetails:
    def __init__(
        self,
        item, edit, baseurl
    ):
        with t.div(_class=cls("p-8")
        ) as doc:
            t.h1("Item", _class=cls("text-2xl", "font-semibold"))
            with t.div(_class=cls("my-6")):
                InventoryItemInfo(item, edit, baseurl)

        self.doc = doc

