import dominate
import dominate.tags as t
from dominate.util import raw

from . import CategoryList, ItemList

from ..helpers import *


class PackageListItemManager:
    def __init__(
        self, categories, items, name=None, description=None, error=False, errormsg=None
    ):
        assert not (error and not errormsg)
        with t.div(id="pkglist-item-manager", _class=cls("p-8", "grid", "grid-cols-4", "gap-3")) as doc:
            with t.div(_class=cls("col-span-1")):
                CategoryList(categories),
            with t.div(_class=cls("col-span-3")):
                ItemList(items),

        self.doc = doc
