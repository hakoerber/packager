import sqlalchemy
from . import app
from .models import *
from .helpers import *

import uuid
import os

import dominate
import dominate.tags as t
from dominate.util import raw

from .components import PackageListManager, NewPackageList, Home

from flask import request, make_response


def get_packagelists():
    return PackageList.query.all()


def get_packagelist_by_id(id):
    return PackageList.query.filter_by(id=str(id)).first()


def add_packagelist(name, description):
    try:
        db.session.add(
            PackageList(id=str(uuid.uuid4()), name=name, description=description)
        )
        db.session.commit()
    except sqlalchemy.exc.IntegrityError:
        db.session.rollback()
        return False


def delete_packagelist(id):
    deletions = PackageList.query.filter_by(id=str(id)).delete()
    if deletions == 0:
        return False
    else:
        db.session.commit()
    return True


@app.route("/")
def root():
    return make_response(
        Home(PackageListManager(get_packagelists()), app.root_path).doc.render(), 200
    )


def is_htmx():
    return request.headers.get("HX-Request") is not None


@app.route("/list/", methods=["POST"])
def add_new_list():
    print(f"headers: {request.headers}")
    name = request.form["name"]
    description = request.form["description"]

    error, errormsg = validate_name(name)

    if not error:
        if add_packagelist(name=name, description=description) is False:
            error = True
            errormsg = f'Name "{name}" already exists'

    if is_htmx():
        return make_response(
            str(
                PackageListManager(
                    get_packagelists(),
                    name=name,
                    description=description,
                    error=error,
                    errormsg=errormsg,
                )
            ),
            200 if error else 201,
        )
    else:
        r = make_response("", 303)
        r.headers["Location"] = "/"
        return r


def validate_name(name):
    error, errormsg = False, None

    if len(name) == 0:
        error = True
        errormsg = f"Name cannot be empty"
    elif name.isspace():
        error = True
        errormsg = f"Name cannot be only whitespace"

    return error, errormsg


@app.route("/list/name/validate", methods=["POST"])
def validate_list_name():
    name = request.form["name"]

    error, errormsg = validate_name(name)

    if not error:
        if PackageList.query.filter_by(name=name).first() is not None:
            error = True
            errormsg = f'Name "{name}" already exists'
    doc = NewPackageList(name=name, error=error, errormsg=errormsg)

    return make_response(doc.render(), 200)


@app.route("/list/<uuid:id>/edit/cancel", methods=["POST"])
def edit_list_cancel(id):
    pkglist = PackageList.query.filter_by(id=str(id)).first()

    with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-200")) as doc:
        t.td(pkglist.name, _class=cls("border", "px-2")),
        t.td(str(pkglist.description), _class=cls("border", "px-2")),
        t.td(
            t.span(_class=cls("mdi", "mdi-delete", "text-xl")),
            id="delete-packagelist",
            data_hx_delete=f"/list/{pkglist.id}",
            _class=cls(
                "border",
                "bg-red-200",
                "hover:bg-red-400",
                "cursor-pointer",
                "w-8",
                "text-center",
            ),
        ),
        t.td(
            t.span(_class=cls("mdi", "mdi-pencil", "text-xl")),
            id="edit-packagelist",
            data_hx_post=f"/list/{pkglist.id}/edit",
            _class=cls(
                "border",
                "bg-blue-200",
                "hover:bg-blue-400",
                "cursor-pointer",
                "w-8",
                "text-center",
            ),
        ),
    return make_response(doc.render(), 200)


@app.route("/list/<uuid:id>/edit/submit", methods=["POST"])
def edit_list_submit(id):
    name = request.form["name"]
    description = request.form["description"]
    if len(name) == 0:
        with t.tr(id="pkglist-edit-row") as doc:
            with t.td(colspan=3, _class=cls("border-none", "bg-purple-100", "h-10")):
                t.p("Name cannot be empty", _class=cls("text-red-400", "text-sm"))
                with t.div(_class=cls("flex", "flex-row", "h-full")):
                    with t.div(
                        _class=cls(
                            "box-border" "border",
                            "border-2",
                            "border-red-500",
                            "bg-purple-100",
                            "mr-1",
                        )
                    ):
                        with t.div(_class=cls("h-full")):
                            t._input(
                                _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                                type="text",
                                name="name",
                                value=name,
                            )
                    with t.div(
                        _class=cls(
                            "border", "border-1", "border-purple-500", "bg-purple-100"
                        )
                    ):
                        t._input(
                            _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                            type="text",
                            name="description",
                            value=description,
                        )
            t.td(
                t.span(_class=cls("mdi", "mdi-cancel", "text-xl")),
                id="edit-packagelist-abort",
                data_hx_post=f"/list/{id}/edit/cancel",
                data_hx_target="#pkglist-edit-row",
                data_hx_swap="outerHTML",
                _class=cls(
                    "border",
                    "bg-red-200",
                    "hover:bg-red-400",
                    "cursor-pointer",
                    "w-8",
                    "text-center",
                ),
            ),
            t.td(
                t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
                id="edit-packagelist-save",
                data_hx_post=f"/list/{id}/edit/submit",
                data_hx_target="#pkglist-edit-row",
                data_hx_swap="outerHTML",
                data_hx_include="closest tr",
                _class=cls(
                    "border",
                    "bg-green-200",
                    "hover:bg-green-400",
                    "cursor-pointer",
                    "w-8",
                    "text-center",
                ),
            ),
        return make_response(doc.render(), 200)

    try:
        pkglist = PackageList.query.filter_by(id=str(id)).first()
        if pkglist is None:
            return make_response("", 404)
        pkglist.name = name
        pkglist.description = description
        try:
            db.session.commit()
        except sqlalchemy.exc.IntegrityError:
            with t.tr(id="pkglist-edit-row") as doc:
                with t.td(
                    colspan=3, _class=cls("border-none", "bg-purple-100", "h-10")
                ):
                    t.p(
                        f"Name {name} already exists",
                        _class=cls("text-red-400", "text-sm"),
                    )
                    with t.div(_class=cls("flex", "flex-row", "h-full")):
                        with t.div(
                            _class=cls(
                                "box-border" "border",
                                "border-2",
                                "border-red-500",
                                "bg-purple-100",
                                "mr-1",
                            )
                        ):
                            with t.div(_class=cls("h-full")):
                                t._input(
                                    _class=cls(
                                        "bg-purple-100", "w-full", "h-full", "px-2"
                                    ),
                                    type="text",
                                    name="name",
                                    value=name,
                                )
                        with t.div(
                            _class=cls(
                                "border",
                                "border-1",
                                "border-purple-500",
                                "bg-purple-100",
                            )
                        ):
                            t._input(
                                _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                                type="text",
                                name="description",
                                value=description,
                            )
                t.td(
                    t.span(_class=cls("mdi", "mdi-cancel", "text-xl")),
                    id="edit-packagelist-abort",
                    data_hx_post=f"/list/{id}/edit/cancel",
                    data_hx_target="#pkglist-edit-row",
                    data_hx_swap="outerHTML",
                    _class=cls(
                        "border",
                        "bg-red-200",
                        "hover:bg-red-400",
                        "cursor-pointer",
                        "w-8",
                        "text-center",
                    ),
                ),
                t.td(
                    t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
                    id="edit-packagelist-save",
                    data_hx_post=f"/list/{id}/edit/submit",
                    data_hx_target="#pkglist-edit-row",
                    data_hx_swap="outerHTML",
                    data_hx_include="closest tr",
                    _class=cls(
                        "border",
                        "bg-green-200",
                        "hover:bg-green-400",
                        "cursor-pointer",
                        "w-8",
                        "text-center",
                    ),
                ),
            return make_response(doc.render(), 200)
    except:
        raise

    with t.tr(_class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-200")) as doc:
        t.td(pkglist.name, _class=cls("border", "px-2")),
        t.td(str(pkglist.description), _class=cls("border", "px-2")),
        t.td(
            t.span(_class=cls("mdi", "mdi-delete", "text-xl")),
            id="delete-packagelist",
            data_hx_delete=f"/list/{pkglist.id}",
            _class=cls(
                "border",
                "bg-red-200",
                "hover:bg-red-400",
                "cursor-pointer",
                "w-8",
                "text-center",
            ),
        ),
        t.td(
            t.span(_class=cls("mdi", "mdi-pencil", "text-xl")),
            id="edit-packagelist",
            data_hx_post=f"/list/{pkglist.id}/edit",
            _class=cls(
                "border",
                "bg-blue-200",
                "hover:bg-blue-400",
                "cursor-pointer",
                "w-8",
                "text-center",
            ),
        ),
    return make_response(doc.render(), 200)


def get_edit_list(pkglist):
    with t.tr(
        _class="h-10",
        id="pkglist-edit-row",
        **{
            "x-data": '{ edit_submit_enabled: document.getElementById("listedit-name").value.trim().length !== 0 }'
        },
    ) as doc:
        with t.td(colspan=3, _class=cls("border-none", "bg-purple-100", "h-full")):
            with t.div(_class=cls("flex", "flex-row", "h-full")):
                with t.div(
                    _class=cls(
                        "border",
                        "border-1",
                        "border-purple-500",
                        "bg-purple-100",
                        "mr-1",
                    )
                ):
                    t._input(
                        _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                        type="text",
                        id="listedit-name",
                        name="name",
                        value=pkglist.name,
                        **{
                            "x-on:input": "edit_submit_enabled = $event.srcElement.value.trim().length !== 0;"
                        },
                    )
                with t.div(
                    _class=cls(
                        "border", "border-1", "border-purple-500", "bg-purple-100"
                    )
                ):
                    t._input(
                        _class=cls("bg-purple-100", "w-full", "h-full", "px-2"),
                        type="text",
                        name="description",
                        value=pkglist.description,
                    )
        t.td(
            t.span(_class=cls("mdi", "mdi-cancel", "text-xl")),
            id="edit-packagelist-abort",
            data_hx_post=f"/list/{pkglist.id}/edit/cancel",
            data_hx_target="#pkglist-edit-row",
            data_hx_swap="outerHTML",
            _class=cls(
                "border",
                "bg-red-200",
                "hover:bg-red-400",
                "cursor-pointer",
                "w-8",
                "text-center",
            ),
        ),
        t.td(
            t.span(_class=cls("mdi", "mdi-content-save", "text-xl")),
            id="edit-packagelist-save",
            data_hx_post=f"/list/{pkglist.id}/edit/submit",
            data_hx_target="#pkglist-edit-row",
            data_hx_swap="outerHTML",
            data_hx_include="closest #pkglist-edit-row",
            _class=cls(
                "border",
                "bg-green-200",
                "hover:bg-green-400",
                "cursor-pointer",
                "w-8",
                "text-center",
            ),
            **{
                "x-bind:class": 'edit_submit_enabled || "cursor-not-allowed opacity-50"',
                "x-on:htmx:before-request": "(e) => edit_submit_enabled || e.preventDefault()",
            },
        ),
    return doc


@app.route("/list/<uuid:id>/edit", methods=["POST"])
def edit_list(id):
    pkglist = get_packagelist_by_id(id)

    return make_response(get_edit_list(pkglist).render(), 200)


@app.route("/list/<uuid:id>", methods=["DELETE"])
def delete_list(id):
    if not delete_packagelist(id=id):
        return make_response("", 404)
    return make_response("", 200)
