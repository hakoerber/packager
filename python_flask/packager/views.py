import sqlalchemy
from . import app
from .models import *
from .helpers import *

import uuid
import os

import dominate
import dominate.tags as t
from dominate.util import raw

from .components import (
    PackageListManager,
    PackageListItemManager,
    NewPackageList,
    Home,
    PackageListTableRowEdit,
    PackageListTableRowNormal,
    PackageListTableRow,
)

from flask import request, make_response


def get_packagelists():
    return PackageList.query.all()


def get_categories():
    return PackageListItemCategory.query.all()


def get_all_items():
    return PackageListItem.query.all()

def get_items(category):
    return PackageListItem.query.filter_by(category_id=str(category.id))

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
    categories = get_categories()
    items = get_all_items()
    error = False
    if not is_htmx():
        edit = request.args.get("edit")
        if edit is not None:
            match = [p for p in packagelists if p.id == edit]
            if match:
                match[0].edit = True
                error = request.args.get("error")
                if error and bool(int(error)):
                    match[0].error = True
                    errormsg = request.args.get("msg")
                    if errormsg:
                        match[0].errormsg = errormsg
                    else:
                        name = request.args.get("name")
                        if name:
                            match[0].errormsg = f"Invalid name: {name}"
                        else:
                            match[0].errormsg = f"Invalid name"

    return make_response(
        Home(PackageListItemManager(categories, items), app.root_path).doc.render(), 200
    )


@app.route("/category/<uuid:id>")
def category(id):
    categories = get_categories()
    print(id)
    for c in categories:
        print(f"{c.id} | {c.name}")
    active_category = [c for c in categories if str(c.id) == str(id)][0]
    items = get_items(active_category)
    error = False
    if not is_htmx():
        edit = request.args.get("edit")
        if edit is not None:
            match = [p for p in packagelists if p.id == edit]
            if match:
                match[0].edit = True
                error = request.args.get("error")
                if error and bool(int(error)):
                    match[0].error = True
                    errormsg = request.args.get("msg")
                    if errormsg:
                        match[0].errormsg = errormsg
                    else:
                        name = request.args.get("name")
                        if name:
                            match[0].errormsg = f"Invalid name: {name}"
                        else:
                            match[0].errormsg = f"Invalid name"

    return make_response(
        Home(PackageListItemManager(categories, items), app.root_path).doc.render(), 200
    )


def is_htmx():
    return request.headers.get("HX-Request") is not None


@app.route("/list/", methods=["POST"])
def add_new_list():
    name = request.form["name"]
    description = request.form["description"]

    error, errormsg = validate_name(name)

    if not error:
        if add_packagelist(name=name, description=description) is False:
            error = True
            errormsg = f'Name "{name}" already exists'

    if is_htmx():
        return make_response(
            PackageListManager(
                get_packagelists(),
                name=name,
                description=description,
                error=error,
                errormsg=errormsg,
            ).doc.render(),
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
    print("cancelling" * 20)
    pkglist = PackageList.query.filter_by(id=str(id)).first()
    return make_response(PackageListTableRowNormal(pkglist).doc.render(), 200)


@app.route("/list/<uuid:id>/edit/submit/", methods=["POST"])
def edit_list_submit(id):
    name = request.form["name"]
    description = request.form["description"]
    error, errormsg = validate_name(name)

    if error:
        if is_htmx():
            return make_response(
                PackageListTableRowEdit(error=True, errormsg=errormsg).doc.render(), 200
            )
        else:
            r = make_response("", 303)
            r.headers["Location"] = f"/?edit={id}&error=1&msg={errormsg}"
            return r

    pkglist = PackageList.query.filter_by(id=str(id)).first()
    if pkglist is None:
        # todo what to do without js?
        return make_response("", 404)

    pkglist.name = name
    pkglist.description = description

    try:
        db.session.commit()
    except sqlalchemy.exc.IntegrityError:
        db.session.rollback()
        errormsg = f'Name "{name}" already exists'
        if is_htmx():
            pkglist.error = True
            pkglist.errormsg = errormsg
            return make_response(PackageListTableRowEdit(pkglist).doc.render(), 200)
        else:
            r = make_response("", 303)
            r.headers["Location"] = f"/?edit={id}&name={name}&error=1&msg={errormsg}"
            return r

    if is_htmx():
        return make_response(PackageListTableRowNormal(pkglist).doc.render(), 200)
    else:
        r = make_response("", 303)
        r.headers["Location"] = "/"
        return r


@app.route("/list/<uuid:id>/edit", methods=["POST"])
def edit_list(id):
    pkglist = get_packagelist_by_id(id)

    out = PackageListTableRowEdit(pkglist).doc
    return make_response(out.render(), 200)


@app.route("/list/<uuid:id>", methods=["DELETE"])
def delete_list(id):
    if not delete_packagelist(id=id):
        return make_response("", 404)
    return make_response("", 200)
