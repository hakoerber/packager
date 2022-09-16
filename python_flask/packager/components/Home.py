import os

import dominate
import dominate.tags as t
from dominate.util import raw

from ..helpers import *


class Home:
    def __init__(self):
        with t.div(id="home", _class=cls("p-8", "max-w-xl")) as doc:
            with t.p():
                t.a("Inventory", href="/inventory/")
            with t.p():
                t.a("Trips", href="/trips/")

        self.doc = doc
