import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


class TripCategoryList:
    def __init__(self, trip, categories):
        with t.div() as doc:
            t.h1("Categories", _class=cls("text-xl", "mb-5"))
            with t.table(
                _class=cls(
                    "table",
                    "table-auto",
                    "border-collapse",
                    "border-spacing-0",
                    "border",
                    "w-full",
                )
            ):
                with t.thead(_class=cls("bg-gray-200")):
                    t.tr(
                        t.th("Name", _class=cls("border", "p-2")),
                        t.th("Weight", _class=cls("border", "p-2")),
                        t.th("Max", _class=cls("border", "p-2")),
                        _class="h-10",
                    )
                with t.tbody() as b:
                    for category in categories:
                        items = [
                            i
                            for i in trip.items
                            if i.inventory_item.category_id == category.id
                        ]
                        biggest_category_weight = 1

                        for cat in categories:
                            category_items = [
                                i
                                for i in trip.items
                                if i.inventory_item.category_id == cat.id
                            ]
                            weight_sum = sum(
                                [
                                    i.inventory_item.weight
                                    for i in category_items
                                    if i.pick
                                ]
                            )
                            if weight_sum > biggest_category_weight:
                                biggest_category_weight = weight_sum

                        weight = sum([i.inventory_item.weight for i in items if i.pick])

                        with t.tr(
                            _class=cls(
                                "h-10",
                                "hover:bg-purple-100",
                                "m-3",
                                "h-full",
                                *["bg-blue-100"]
                                if category.active
                                else (
                                    ["bg-red-100"]
                                    if any([i.pick != i.pack for i in items])
                                    else []
                                ),
                            )
                        ) as doc:
                            with t.td(
                                _class=cls(
                                    "border",
                                    "p-0",
                                    "m-0",
                                    *["font-bold"] if category.active else [],
                                )
                            ):
                                t.a(
                                    category.name,
                                    id="select-category",
                                    # data_hx_post=f"/list/{pkglist.id}/edit",
                                    href=f"/trip/{trip.id}/category/{category.id}",
                                    # data_hx_target="closest tr",
                                    # data_hx_swap="outerHTML",
                                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                                )

                            with t.td(
                                _class=cls("border", "p-0", "m-0"),
                                style="position:relative;",
                            ):
                                with t.a(
                                    id="select-category",
                                    # data_hx_post=f"/list/{pkglist.id}/edit",
                                    href=f"/category/{category.id}",
                                    # data_hx_target="closest tr",
                                    # data_hx_swap="outerHTML",
                                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                                ):
                                    width = int(weight / biggest_category_weight * 100)
                                    t.p(weight)
                                t.div(
                                    _class=cls("bg-blue-600", "h-1.5"),
                                    style=f"width: {width}%;position:absolute;left:0;bottom:0;right:0;",
                                )
                            with t.td(_class=cls("border", "p-0", "m-0")):
                                t.a(
                                    sum([i.inventory_item.weight for i in items]),
                                    id="select-category",
                                    # data_hx_post=f"/list/{pkglist.id}/edit",
                                    href=f"/category/{category.id}",
                                    # data_hx_target="closest tr",
                                    # data_hx_swap="outerHTML",
                                    _class=cls("inline-block", "p-2", "m-0", "w-full"),
                                )
                # with t.tr(_class=cls("h-10", "hover:bg-purple-200", "bg-gray-300", "font-bold")) as doc:
                #     with t.td(_class=cls("border", "p-0", "m-0")):
                #         t.a(
                #             "Sum",
                #             _class=cls("block", "p-2", "m-2"),
                #         )
                #     with t.td(_class=cls("border", "p-0", "m-0")):
                #         t.a(
                #             sum(([i.inventory_item.weight for i in c.items]) for c in categories]),
                #             _class=cls("block", "p-2", "m-2"),
                #         )
                #     with t.td(_class=cls("border", "p-0", "m-0", "font-normal")):
                #         t.a(
                #             sum(([i.inventory_item.weight for i in c.items]) for c in categories]),
                #             _class=cls("block", "p-2", "m-2"),
                #         )

        self.doc = doc
