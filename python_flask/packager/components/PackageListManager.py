import dominate
import dominate.tags as t
from dominate.util import raw

from . import NewPackageList, PackageListTable

from ..helpers import *


class PackageListManager:
    def __init__(
        self, pkglists, name=None, description=None, error=False, errormsg=None
    ):
        assert not (error and not errormsg)
        with t.div(
            id="pkglist-manager",
            _class=cls("p-8", "max-w-xl"),
            **{
                "x-data": '{ submit_enabled: document.getElementById("listname").value.trim().length !== 0 }'
            },
        ) as doc:
            PackageListTable(pkglists),
            NewPackageList(
                name=name, description=description, error=error, errormsg=errormsg
            )

        self.doc = doc
