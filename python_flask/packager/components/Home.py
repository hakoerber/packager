import os

import dominate
import dominate.tags as t
from dominate.util import raw


class Home:
    def __init__(self, element, root_path):
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
        doc.add(element.doc)
        self.doc = doc
