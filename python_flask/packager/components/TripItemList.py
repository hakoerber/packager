import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


class TripItemRowEdit(object):
    def __init__(self, item, biggest_item_weight):
        with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-100")) as doc:
            with t.td(_class=cls("border")):
                with t.a(
                    id="select-category",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"?item_{'unpick' if item.pick else 'pick'}={item.inventory_item.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls(
                        "inline-block",
                        "p-2",
                        "m-0",
                        "w-full",
                        "flex",
                        "justify-center",
                        "content-center",
                    ),
                ):
                    t._input(
                        type="checkbox", **({"checked": True} if item.pick else {})
                    )
            with t.td(_class=cls("border")):
                with t.a(
                    id="select-category",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"?item_{'unpack' if item.pack else 'pack'}={item.inventory_item.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls(
                        "inline-block",
                        "p-2",
                        "m-0",
                        "w-full",
                        "flex",
                        "justify-center",
                        "content-center",
                    ),
                ):
                    t._input(
                        type="checkbox", **({"checked": True} if item.pack else {})
                    )
            # with t.td(item.name, _class=cls("border", "px-2")),
            with t.td(colspan=2, _class=cls("border-none", "bg-purple-100", "h-10")):
                with t.div():
                    t.form(
                        id="edit-item",
                        action=f"/inventory/item/{item.inventory_item.id}/edit/submit/",
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


class TripItemRowNormal(object):
    def __init__(self, item, biggest_item_weight):
        with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-100")) as doc:
            with t.td(_class=cls("border")):
                with t.a(
                    id="select-category",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"?item_{'unpick' if item.pick else 'pick'}={item.inventory_item.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls(
                        "inline-block",
                        "p-2",
                        "m-0",
                        "w-full",
                        "justify-center",
                        "content-center",
                        "flex",
                    ),
                ):
                    t._input(
                        type="checkbox",
                        form=f"toggle-item-pick",
                        name="pick-{item.inventory_item.id}",
                        **({"checked": True} if item.pick else {}),
                    )  # , xstyle="position: relative;z-index: 1;pointer-events: auto; ")
            with t.td(_class=cls("border")):
                with t.a(
                    id="select-category",
                    # data_hx_post=f"/list/{pkglist.id}/edit",
                    href=f"?item_{'unpack' if item.pack else 'pack'}={item.inventory_item.id}",
                    # data_hx_target="closest tr",
                    # data_hx_swap="outerHTML",
                    _class=cls(
                        "inline-block",
                        "p-2",
                        "m-0",
                        "w-full",
                        "flex",
                        "justify-center",
                        "content-center",
                    ),
                ):
                    t._input(
                        type="checkbox",
                        form=f"toggle-item-pack",
                        name="pack-{item.inventory_item.id}",
                        **({"checked": True} if item.pack else {}),
                    )  # , xstyle="position: relative;z-index: 1;pointer-events: auto; ")
            t.td(
                item.inventory_item.name,
                _class=cls(
                    "border",
                    "px-2",
                    *(("bg-red-100",) if item.pick != item.pack else ()),
                ),
            ),
            width = int(item.inventory_item.weight / biggest_item_weight * 100)
            with t.td(_class=cls("border", "px-2"), style="position:relative;"):
                t.p(str(item.inventory_item.weight))
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
        self.doc = doc


class TripItemRow(object):
    def __init__(self, item, biggest_item_weight):
        if item.edit:
            self.doc = TripItemRowEdit(item, biggest_item_weight)
        else:
            self.doc = TripItemRowNormal(item, biggest_item_weight)


class TripItemList:
    def __init__(self, trip, active_category):
        with t.div() as doc:
            t.h1("Items", _class=cls("text-xl", "mb-5"))
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
                        t.th("Take?", _class=cls("border", "p-2", "w-0")),
                        t.th("Packed?", _class=cls("border", "p-2", "w-0")),
                        t.th("Name", _class=cls("border", "p-2")),
                        t.th("Weight", _class=cls("border", "p-2")),
                        _class="h-10",
                    )
                with t.tbody() as b:
                    if active_category:
                        biggest_item_weight = max(
                            [
                                i.inventory_item.weight
                                for i in trip.items
                                if i.inventory_item.category_id == active_category.id
                            ]
                            or [0]
                        )
                    else:
                        biggest_item_weight = max(
                            [i.inventory_item.weight for i in trip.items] or [0]
                        )
                    if biggest_item_weight <= 0:
                        biggest_item_weight = 1
                    print(active_category)
                    for item in trip.items:
                        if (
                            not active_category
                            or item.inventory_item.category_id == active_category.id
                        ):
                            TripItemRow(item, biggest_item_weight).doc

        self.doc = doc
