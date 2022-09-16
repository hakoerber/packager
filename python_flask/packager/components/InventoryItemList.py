import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


class InventoryItemRowEdit(object):
    def __init__(self, item, biggest_item_weight):
        with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-100")) as doc:
            with t.td(colspan=2, _class=cls("border-none", "bg-purple-100", "h-10")):
                with t.div():
                    t.form(
                        id="edit-item",
                        action=f"/inventory/item/{item.id}/edit/submit/",
                        target="_self",
                        method="post",
                        # data_hx_post=f"/list/{pkglist.id}/edit/submit",
                        # data_hx_target="closest tr",
                        # data_hx_swap="outerHTML",
                    )
                with t.div(_class=cls("flex", "flex-row", "h-full")):
                    with t.span(
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
                            id="item-edit-name",
                            form="edit-item",
                            name="name",
                            value=item.name,
                            **{
                                "x-on:input": "edit_submit_enabled = $event.srcElement.value.trim().length !== 0;"
                            },
                        )
                    with t.span(
                        _class=cls(
                            "border", "border-1", "border-purple-500", "bg-purple-100"
                        )
                    ):
                        t._input(
                            _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                            type="text",
                            id="item-edit-weight",
                            name="weight",
                            form="edit-item",
                            value=item.weight,
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
                    id="edit-item-abort",
                ):
                    with t.a(
                        href=f"/inventory/category/{item.category.id}",
                        # data_hx_post=f"/item/{item.id}/edit/cancel",
                        # data_hx_target="closest tr",
                        # data_hx_swap="outerHTML",
                    ):
                        with t.button():
                            t.span(_class=cls("mdi", "mdi-cancel", "text-xl")),
                with t.td(
                    id="edit-item-save",
                    _class=cls(
                        "border",
                        "bg-green-200",
                        "hover:bg-green-400",
                        "cursor-pointer",
                        "w-8",
                        "text-center",
                    ),
                    # **{
                    #     "x-bind:class": 'edit_submit_enabled || "cursor-not-allowed opacity-50"',
                    #     "x-on:htmx:before-request": "(e) => edit_submit_enabled || e.preventDefault()",
                    # },
                ):
                    with t.button(type="submit", form="edit-item"):
                        t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
        self.doc = doc


class InventoryItemRowNormal(object):
    def __init__(self, item, biggest_item_weight):
        with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-100")) as doc:
            with t.td(_class=cls("border", "p-0", "m-0")):
                t.a(
                    item.name,
                    href=f"/inventory/item/{item.id}/",
                    _class=cls("p-2", "w-full", "inline-block"),
                )
            width = int(item.weight / biggest_item_weight * 100)
            with t.td(_class=cls("border", "px-2"), style="position:relative;"):
                t.p(str(item.weight))
                t.div(
                    _class=cls("bg-blue-600", "h-1.5"),
                    style=";".join(
                        [
                            f"width: {width}%",
                            "position:absolute",
                            "left:0",
                            "bottom:0",
                            "right:0",
                        ]
                    ),
                )
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
                    # data_hx_post=f"/item/{item.id}/edit",
                    href=f"?edit={item.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    id="start-edit-item",
                ):
                    with t.button():
                        t.span(_class=cls("mdi", "mdi-pencil", "text-xl")),
            with t.td(
                _class=cls(
                    "border",
                    "bg-red-200",
                    "hover:bg-red-400",
                    "cursor-pointer",
                    "w-8",
                    "text-center",
                )
            ):
                with t.a(
                    # data_hx_delete=f"/item/{item.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    href=f"/inventory/item/{item.id}/delete"
                ):
                    with t.button():
                        t.span(_class=cls("mdi", "mdi-delete", "text-xl"))
        self.doc = doc


class InventoryItemRow(object):
    def __init__(self, item, biggest_item_weight):
        if item.edit:
            self.doc = InventoryItemRowEdit(item, biggest_item_weight)
        else:
            self.doc = InventoryItemRowNormal(item, biggest_item_weight)


def InventoryItemList(items):
    with t.div() as doc:
        t.h1("Items", _class=cls("text-2xl", "mb-5"))
        with t.table(
            id="item-table",
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
                    t.th("Weight", _class=cls("border", "p-2")),
                    _class="h-10",
                )
            with t.tbody() as b:
                biggest_item_weight = max([i.weight for i in items] or [0])
                if biggest_item_weight <= 0:
                    biggest_item_weight = 1
                for item in items:
                    InventoryItemRow(item, biggest_item_weight).doc

    return doc
