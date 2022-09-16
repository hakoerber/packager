import dominate
import dominate.tags as t
from dominate.util import raw

from .InventoryItemList import InventoryItemList
from .InventoryCategoryList import InventoryCategoryList
from .InventoryNewItem import InventoryNewItem

from ..helpers import *


class InventoryItemManager:
    def __init__(
        self,
        categories,
        items,
        active_category,
        name=None,
        description=None,
        error=False,
        errormsg=None,
    ):
        assert not (error and not errormsg)
        with t.div(
            id="pkglist-item-manager", _class=cls("p-8", "grid", "grid-cols-4", "gap-3")
        ) as doc:
            with t.div(_class=cls("col-span-2")):
                InventoryCategoryList(categories),
            with t.div(_class=cls("col-span-2")):
                InventoryItemList(items),
                InventoryNewItem(categories, active_category)

        self.doc = doc
