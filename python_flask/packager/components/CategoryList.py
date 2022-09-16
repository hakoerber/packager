import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *

def CategoryList(categories):
    with t.div(id="packagelist-table") as doc:
        t.h1("Categories", _class=cls("text-2xl", "mb-5"))
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
            with t.tbody() as b:
                for category in categories:
                    with t.tr(_class=cls("h-10", "hover:bg-purple-200")) as doc:
                        with t.td(_class=cls("border", "p-0", "m-0")):
                            t.a(
                                category.name,
                                id="select-category",
                                # data_hx_post=f"/list/{pkglist.id}/edit",
                                href=f"/category/{category.id}",
                                # data_hx_target="closest tr",
                                # data_hx_swap="outerHTML",
                                _class=cls("block", "p-2", "m-2"),
                            )

    return doc
