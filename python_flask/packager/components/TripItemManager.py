import dominate
import dominate.tags as t
from dominate.util import raw

from .TripCategoryList import TripCategoryList
from .TripItemList import TripItemList

from ..helpers import *


class TripItemManager:
    def __init__(self, trip, categories, active_category):
        with t.div(
            id="pkglist-item-manager", _class=cls("grid", "grid-cols-4", "gap-3")
        ) as doc:
            with t.div(_class=cls("col-span-2")):
                TripCategoryList(trip, categories),
            with t.div(_class=cls("col-span-2")):
                TripItemList(trip, active_category=active_category),

        self.doc = doc
