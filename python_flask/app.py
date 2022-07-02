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


def add_packagelist(name, description):
    try:
        db.session.add(
            PackageList(id=str(uuid.uuid4()), name=name, description=description)
        )
        db.session.commit()
    except sqlalchemy.exc.IntegrityError:
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
        t.h1("Package Lists", _class=style("text-2xl", "mb-5"))
        with t.table(
            id="packagelist-table",
            _class=style("table", "border-collapse", "border", "w-full"),
        ):
            with t.thead(_class=style("bg-gray-200")):
                t.tr(
                    t.th("Name", _class=style("border", "p-2")),
                    t.th("Description", _class=style("border", "p-2")),
                    t.th(_class=style("border p-2")),
                )
            with t.tbody():
                for pkglist in pkglists:
                    t.tr(
                        t.td(pkglist.name, _class=style("border", "p-2")),
                        t.td(str(pkglist.description), _class=style("border", "p-2")),
                        t.td(
                            "x",
                            id="delete-packagelist",
                            data_hx_delete=f"/list/{pkglist.id}",
                            data_hx_target="#packagelist-table",
                            data_hx_swap="outerHTML",
                            _class=style(
                                "border",
                                "bg-red-200",
                                "min-w-max",
                                "hover:bg-red-200",
                                "cursor-pointer",
                                "w-8",
                                "text-center",
                            ),
                        ),
                        _class=style("even:bg-gray-100", "hover:bg-purple-200"),
                    )

    return doc


def style(*args):
    return " ".join(args)


@app.route("/")
def root():
    doc = dominate.document(title="My cool title")
    with doc.head:
        t.script(src="https://unpkg.com/htmx.org@1.7.0")
        t.script(src="https://cdn.tailwindcss.com")
        t.link(
            rel="stylesheet",
            href="https://cdn.jsdelivr.net/npm/@mdi/font@6.9.96/css/materialdesignicons.min.css",
        )
    with doc:
        with t.div(_class=style("p-8", "max-w-xl")):
            t.script(raw(open("app.js").read()))
            pkglist_table()

            with t.form(
                name="new_pkglist",
                data_hx_post="/list/",
                data_hx_target="#packagelist-table",
                data_hx_swap="outerHTML",
                _class=style("mt-8", "p-5", "border-2", "border-gray-200"),
            ):
                with t.div(_class=style("mb-5", "flex", "flex-row", "items-center")):
                    t.span(_class=style("mdi", "mdi-playlist-plus", "text-2xl", "mr-4"))
                    t.p("Add new package list", _class=style("inline", "text-xl"))
                with t.div(_class=style("w-11/12", "mx-auto")):
                    with t.div(
                        _class=style(
                            "flex", "flex-row", "justify-center", "items-center", "pb-8"
                        )
                    ):
                        t.label(
                            "Name",
                            _for="listname",
                            _class=style("font-bold", "w-1/2", "text-center"),
                        )
                        t._input(
                            type="text",
                            id="listname",
                            name="name",
                            value="",
                            _class=style(
                                "block",
                                "w-1/2",
                                "p-2",
                                "bg-gray-100",
                                "appearance-none",
                                "border-2",
                                "border-gray-300",
                                "rounded",
                                "focus:outline-none",
                                "focus:bg-white",
                                "focus:border-purple-500",
                            ),
                        )
                    with t.div(
                        _class=style(
                            "flex", "flex-row", "justify-center", "items-center", "pb-8"
                        )
                    ):
                        t.label(
                            "Description",
                            _for="listdesc",
                            _class=style("font-bold", "w-1/2", "text-center"),
                        )
                        t._input(
                            type="text",
                            id="listdesc",
                            name="description",
                            value="",
                            _class=style(
                                "block",
                                "w-1/2",
                                "p-2",
                                "bg-gray-100",
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
                        _class=style(
                            "py-2",
                            "border-2",
                            "rounded",
                            "border-gray-300",
                            "mx-auto",
                            "w-full",
                            "hover:border-purple-500",
                            "hover:bg-purple-200",
                        ),
                    )

    return doc.render()


@app.route("/list/", methods=["POST"])
def add_new_list():
    print(request.form)
    name = request.form["name"]
    description = request.form["description"]
    if add_packagelist(name=name, description=description) is False:
        return make_response(f'A package list with name "{name}" already exists', 400)

    return pkglist_table().render()


@app.route("/list/<uuid:id>", methods=["DELETE"])
def delete_list(id):
    if not delete_packagelist(id=id):
        return make_response("", 404)
    return pkglist_table().render()
