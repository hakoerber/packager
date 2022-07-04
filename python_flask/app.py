import uuid
import sqlalchemy
from flask import Flask, request, make_response

from flask_sqlalchemy import SQLAlchemy

import dominate
import dominate.tags as t
from dominate.util import raw


app = Flask(__name__)
app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///./db.sqlite"
app.config["SQLALCHEMY_TRACK_MODIFICATIONS"] = False
db = SQLAlchemy(app)


class PackageList(db.Model):
    id = db.Column(db.String(36), primary_key=True)
    name = db.Column(db.Text, unique=True)
    description = db.Column(db.Text)


db.create_all()
try:
    db.session.add(
        PackageList(
            id="ab2f16c2-d5f5-460b-b149-0fc9eec12887",
            name="EDC",
            description="What you always carry",
        )
    )
    db.session.add(
        PackageList(
            id="9f3a72cd-7e30-4263-bd52-92fb7bed1242",
            name="Camping",
            description="For outdoors",
        )
    )

    db.session.commit()
except sqlalchemy.exc.IntegrityError:
    pass


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


def pkglist_table():
    pkglists = get_packagelists()
    doc = t.div(id="packagelist-table")
    with doc:
        t.h1("Package Lists", _class=cls("text-2xl", "mb-5"))
        with t.table(
            id="packagelist-table",
            _class=cls(
                "table",
                "table-auto",
                # "border-separate",
                "border-collapse",
                "border-spacing-0",
                "border",
                "w-full",
            ),
        ):
            with t.thead(_class=cls("bg-gray-200")):
                t.tr(
                    t.th("Name", _class=cls("border", "p-2")),
                    t.th("Description", _class=cls("border", "p-2")),
                    t.th(_class=cls("border p-2")),
                    t.th(_class=cls("border p-2")),
                    _class="h-10",
                )
            with t.tbody(data_hx_target="closest tr", data_hx_swap="outerHTML"):
                for pkglist in pkglists:
                    t.tr(
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
                        _class=cls("h-10", "even:bg-gray-100", "hover:bg-purple-200"),
                    )

    return doc


def cls(*args):
    return " ".join([a for a in args if a is not None])


def new_pkglist_form(name=None, description=None, error=False, errormsg=None):
    assert not (error and not errormsg)
    with t.form(
        id="new-pkglist",
        name="new_pkglist",
        data_hx_post="/list/",
        data_hx_target="#pkglist-manager",
        data_hx_swap="outerHTML",
        _class=cls("mt-8", "p-5", "border-2", "border-gray-200"),
        **{"x-on:htmx:before-request": "(e) => submit_enabled || e.preventDefault()"},
    ) as doc:
        with t.div(_class=cls("mb-5", "flex", "flex-row", "items-center")):
            t.span(_class=cls("mdi", "mdi-playlist-plus", "text-2xl", "mr-4"))
            t.p("Add new package list", _class=cls("inline", "text-xl"))
        with t.div(_class=cls("w-11/12", "mx-auto")):
            with t.div(_class=cls("pb-8")):
                with t.div(
                    _class=cls("flex", "flex-row", "justify-center", "items-start")
                ):
                    t.label(
                        "Name",
                        _for="listname",
                        _class=cls("font-bold", "w-1/2", "p-2", "text-center"),
                    )
                    with t.div(_class=cls("w-1/2")):
                        t._input(
                            type="text",
                            id="listname",
                            name="name",
                            **{"value": name} if name is not None else {},
                            data_hx_target="#new-pkglist",
                            data_hx_post="/list/name/validate",
                            data_hx_swap="outerHTML",
                            _class=cls(
                                "block",
                                "w-full",
                                "p-2",
                                "bg-gray-50",
                                "appearance-none" if error else None,
                                "border-2",
                                "border-red-400" if error else "border-gray-300",
                                "rounded",
                                "focus:outline-none",
                                "focus:bg-white",
                                "focus:border-purple-500" if not error else None,
                            ),
                            **{
                                "x-on:input": "submit_enabled = $event.srcElement.value.trim().length !== 0;"
                            },
                        )
                        t.p(
                            errormsg, _class=cls("mt-1", "text-red-400", "text-sm")
                        ) if error else None
            with t.div(
                _class=cls("flex", "flex-row", "justify-center", "items-center", "pb-8")
            ):
                t.label(
                    "Description",
                    _for="listdesc",
                    _class=cls("font-bold", "w-1/2", "text-center"),
                )
                t._input(
                    type="text",
                    id="listdesc",
                    name="description",
                    **{"value": description} if description is not None else {},
                    _class=cls(
                        "block",
                        "w-1/2",
                        "p-2",
                        "bg-gray-50",
                        "appearance-none",
                        "border-2",
                        "border-gray-300",
                        "rounded",
                        "focus:outline-none",
                        "focus:bg-white",
                        "focus:border-purple-500",
                    ),
                )
            t._input(
                type="submit",
                value="Add",
                **{
                    "x-bind:class": 'submit_enabled ? "" : "cursor-not-allowed opacity-50"'
                },
                _class=cls(
                    "py-2",
                    "border-2",
                    "rounded",
                    "border-gray-300",
                    "mx-auto",
                    "w-full",
                    "hover:border-purple-500" if not error else None,
                    "hover:bg-purple-200" if not error else None,
                ),
            )
    return doc


def pkglist_manager(name=None, description=None, error=False, errormsg=None):
    assert not (error and not errormsg)
    with t.div(
        id="pkglist-manager",
        _class=cls("p-8", "max-w-xl"),
        **{
            "x-data": '{ submit_enabled: document.getElementById("listname").value.trim().length !== 0 }'
        },
    ) as doc:
        pkglist_table()
        new_pkglist_form(
            name=name, description=description, error=error, errormsg=errormsg
        )
    return doc


@app.route("/")
def root():
    doc = dominate.document(title="My cool title")
    with doc.head:
        t.script(src="https://unpkg.com/htmx.org@1.7.0")
        t.script(src="https://cdn.tailwindcss.com")
        t.script(src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js", defer=True)
        t.link(
            rel="stylesheet",
            href="https://cdn.jsdelivr.net/npm/@mdi/font@6.9.96/css/materialdesignicons.min.css",
        )
    with doc:
        t.script(raw(open("app.js").read()))
        pkglist_manager()

    return make_response(doc.render(), 200)


@app.route("/list/", methods=["POST"])
def add_new_list():
    name = request.form["name"]
    description = request.form["description"]
    error, errormsg = validate_name(name)

    print(error, errormsg)
    if not error:
        if add_packagelist(name=name, description=description) is False:
            error = True
            errormsg = f'Name "{name}" already exists'

    return make_response(
        pkglist_manager(
            name=name, description=description, error=error, errormsg=errormsg
        ).render(),
        200,
    )


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
    doc = new_pkglist_form(name=name, error=error, errormsg=errormsg)

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
            with t.td(colspan=2, _class=cls("border-none", "bg-purple-100", "h-10")):
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
                    colspan=2, _class=cls("border-none", "bg-purple-100", "h-10")
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
            "x-data": '{ edit_submit_enabled: document.getElementById("listedit-name").value.trim().length() !== 0 }'
        },
    ) as doc:
        with t.td(colspan=2, _class=cls("border-none", "bg-purple-100", "h-full")):
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
