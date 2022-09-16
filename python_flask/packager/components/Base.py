import os

import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


class Base:
    def __init__(self, element, root_path, active_page=None):
        doc = dominate.document(title="Packager")
        with doc.head:
            t.script(src="https://unpkg.com/htmx.org@1.7.0")
            t.script(src="https://cdn.tailwindcss.com")
            t.script(src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.js", defer=True)
            t.link(
                rel="stylesheet",
                href="https://cdn.jsdelivr.net/npm/@mdi/font@6.9.96/css/materialdesignicons.min.css",
            )
        with doc:
            t.script(raw(open(os.path.join(root_path, "js/app.js")).read()))
            with t.header(
                _class=cls(
                    "bg-gray-200",
                    "p-5",
                    "flex",
                    "flex-row",
                    "flex-nowrap",
                    "justify-between",
                    "items-center",
                )
            ):
                t.span("Packager", _class=cls("text-xl", "font-semibold"))
                with t.nav(
                    _class=cls("grow", "flex", "flex-row", "justify-center", "gap-x-6")
                ):
                    basecls = ["text-lg"]
                    activecls = ["font-bold", "underline"]
                    t.a(
                        "Inventory",
                        href="/inventory/",
                        _class=cls(
                            *basecls, *(activecls if active_page == "inventory" else [])
                        ),
                    )
                    t.a(
                        "Trips",
                        href="/trips/",
                        _class=cls(
                            *basecls, *(activecls if active_page == "trips" else [])
                        ),
                    )
        doc.add(element.doc)
        self.doc = doc
