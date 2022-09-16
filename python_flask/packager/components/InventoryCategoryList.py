import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


def InventoryCategoryList(categories):
    with t.div() as doc:
        t.h1("Categories", _class=cls("text-2xl", "mb-5"))
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
                    _class="h-10",
                )
            with t.tbody() as b:
                biggest_category_weight = max(
                    [sum([i.weight for i in c.items]) for c in categories] or [0]
                )
                for category in categories:
                    with t.tr(
                        _class=cls("h-10", "hover:bg-purple-100", "m-3", "h-full")
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
                                href=f"/inventory/category/{category.id}",
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
                                href=f"/inventory/category/{category.id}",
                                # data_hx_target="closest tr",
                                # data_hx_swap="outerHTML",
                                _class=cls("inline-block", "p-2", "m-0", "w-full"),
                            ):
                                weight = sum([i.weight for i in category.items])
                                width = int(weight / biggest_category_weight * 100)
                                t.p(weight)
                            t.div(
                                _class=cls("bg-blue-600", "h-1.5"),
                                style=f"width: {width}%;position:absolute;left:0;bottom:0;right:0;",
                            )
                            # t.progress(max=biggest_category_weight, value=weight)
                with t.tr(
                    _class=cls(
                        "h-10", "hover:bg-purple-200", "bg-gray-300", "font-bold"
                    )
                ) as doc:
                    with t.td(_class=cls("border", "p-0", "m-0")):
                        t.a("Sum", _class=cls("block", "p-2", "m-2"))
                    with t.td(_class=cls("border", "p-0", "m-0")):
                        t.a(
                            sum([sum([i.weight for i in c.items]) for c in categories]),
                            _class=cls("block", "p-2", "m-2"),
                        )

    return doc
