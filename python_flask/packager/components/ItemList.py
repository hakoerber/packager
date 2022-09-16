import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *

def ItemList(items):
    with t.div(id="packagelist-table") as doc:
        t.h1("Items", _class=cls("text-2xl", "mb-5"))
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
                    t.th("Weight", _class=cls("border", "p-2")),
                    _class="h-10",
                )
            with t.tbody() as b:
                for item in items:
                    with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-200")) as doc:
                        t.td(item.name, _class=cls("border", "px-2")),
                        t.td(str(item.weight), _class=cls("border", "px-2")),

    return doc
