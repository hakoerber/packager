import dominate
import dominate.tags as t
from dominate.util import raw

from .TripTable import TripTable
from .NewTrip import NewTrip

from ..helpers import *


class TripList:
    def __init__(self, trips):
        with t.div(id="trips-manager", _class=cls("p-8")) as doc:
            TripTable(trips),
            NewTrip(
                # name=name, description=description, error=error, errormsg=errormsg
            )

        self.doc = doc
