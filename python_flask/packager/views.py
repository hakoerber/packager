import sqlalchemy
from . import app
from .models import *
from .helpers import *

import uuid
import os
import urllib
import datetime

import dominate
import dominate.tags as t
from dominate.util import raw

from .components import (
    InventoryItemManager,
    Base,
    Home,
    TripList,
    TripManager,
    InventoryItemDetails,
)

from flask import request, make_response


def get_categories():
    return InventoryItemCategory.query.all()


def get_trips():
    return Trip.query.all()


def get_all_items():
    return InventoryItem.query.all()


def get_triptypes():
    return TripType.query.all()


def get_items(category):
    return InventoryItem.query.filter_by(category_id=str(category.id))


def get_item(id):
    return InventoryItem.query.filter_by(id=str(id)).first()


def get_trip(id):
    return Trip.query.filter_by(id=str(id)).first()


def pick_item(trip_id, item_id):
    item = TripItems.query.filter_by(item_id=str(item_id), trip_id=str(trip_id)).first()
    item.pick = True
    db.session.commit()


def unpick_item(trip_id, item_id):
    item = TripItems.query.filter_by(item_id=str(item_id), trip_id=str(trip_id)).first()
    item.pick = False
    db.session.commit()


def pack_item(trip_id, item_id):
    item = TripItems.query.filter_by(item_id=str(item_id), trip_id=str(trip_id)).first()
    item.pack = True
    db.session.commit()


def unpack_item(trip_id, item_id):
    item = TripItems.query.filter_by(item_id=str(item_id), trip_id=str(trip_id)).first()
    item.pack = False
    db.session.commit()


def add_item(name, weight, category_id):
    db.session.add(
        InventoryItem(
            id=str(uuid.uuid4()),
            name=name,
            description="",
            weight=weight,
            category_id=category_id,
        )
    )
    db.session.commit()


@app.route("/")
def root():
    return make_response(Base(Home(), app.root_path).doc.render(), 200)


@app.route("/inventory/")
def inventory():
    categories = get_categories()
    items = get_all_items()

    args = request.args.to_dict()

    error = False
    if not is_htmx():
        edit = request.args.get("edit")
        if edit is not None:
            match = [i for i in items if i.id == edit]
            if match:
                match[0].edit = True

    return make_response(
        Base(
            InventoryItemManager(categories, items, active_category=None),
            app.root_path,
            active_page="inventory",
        ).doc.render(),
        200,
    )


@app.route("/inventory/item/<uuid:id>/")
def inventory_item(id):
    item = get_item(id)

    args = request.args.to_dict()
    edit = args.pop("edit", None)

    return make_response(
        Base(
            InventoryItemDetails(item, edit=edit, baseurl=f"/inventory/item/{id}/",), app.root_path, active_page="inventory"
        ).doc.render(),
        200,
    )

@app.route(
    "/inventory/item/<uuid:id>/edit/<string:attribute>/submit/",
    methods=["POST"],
)
def edit_inventory_submit(id, attribute):
    new_value = request.form[attribute]

    if attribute in ("weight"):
        new_value = int(new_value)

    updates = InventoryItem.query.filter_by(id=str(id)).update({attribute: new_value})
    db.session.commit()
    if updates == 0:
        # todo what to do without js?
        return make_response("", 404)

    redirect = request.path[: -(len(f"edit/{attribute}/submit/"))]

    r = make_response("", 303)
    r.headers["Location"] = redirect
    return r

@app.route("/trips/")
def trips():
    trips = get_trips()

    return make_response(
        Base(TripList(trips), app.root_path, active_page="trips").doc.render(), 200
    )


@app.route("/trip/<uuid:id>/")
def trip(id):

    args = request.args.to_dict()

    item_to_pick = args.pop("item_pick", None)
    if item_to_pick:
        pick_item(id, item_to_pick)
        r = make_response("", 303)
        if args:
            r.headers["Location"] = f"/trip/{id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/"
        return r

    item_to_unpick = args.pop("item_unpick", None)
    if item_to_unpick:
        unpick_item(id, item_to_unpick)
        r = make_response("", 303)
        if args:
            r.headers["Location"] = f"/trip/{id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/"
        return r

    item_to_pack = args.pop("item_pack", None)
    if item_to_pack:
        pack_item(id, item_to_pack)
        r = make_response("", 303)
        if args:
            r.headers["Location"] = f"/trip/{id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/"
        return r

    item_to_unpack = args.pop("item_unpack", None)
    if item_to_unpack:
        unpack_item(id, item_to_unpack)
        r = make_response("", 303)
        if args:
            r.headers["Location"] = f"/trip/{id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/"
        return r

    type_to_add = args.pop("type_add", None)
    if type_to_add:
        newtype = TripToTripType(trip_id=str(id), trip_type_id=str(type_to_add))
        db.session.add(newtype)
        db.session.commit()
        r = make_response("", 303)
        if args:
            r.headers["Location"] = f"/trip/{id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/"
        return r

    type_to_remove = args.pop("type_remove", None)
    if type_to_remove:
        newtype = TripToTripType.query.filter_by(
            trip_id=str(id), trip_type_id=str(type_to_remove)
        ).delete()
        db.session.commit()
        r = make_response("", 303)
        if args:
            r.headers["Location"] = f"/trip/{id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/"
        return r

    trip = get_trip(id)

    items = get_all_items()

    categories = get_categories()

    edit = args.pop("edit", None)

    for item in items:
        try:
            db.session.add(
                TripItems(trip_id=str(id), item_id=str(item.id), pick=False, pack=False)
            )
            db.session.commit()
        except sqlalchemy.exc.IntegrityError:
            db.session.rollback()

    return make_response(
        Base(
            TripManager(
                trip,
                categories=categories,
                active_category=None,
                edit=edit,
                baseurl=f"/trip/{id}/",
                triptypes=get_triptypes(),
            ),
            app.root_path,
            active_page="trips",
        ).doc.render(),
        200,
    )


@app.route(
    "/trip/<uuid:id>/category/<uuid:category_id>/edit/<string:attribute>/submit/",
    methods=["POST"],
)
@app.route("/trip/<uuid:id>/edit/<string:attribute>/submit/", methods=["POST"])
def edit_trip_category_location_edit_submit(id, category_id=None, attribute=None):
    new_value = request.form[attribute]

    if attribute in ("start_date", "end_date"):
        new_value = datetime.date.fromisoformat(new_value)

    updates = Trip.query.filter_by(id=str(id)).update({attribute: new_value})
    db.session.commit()
    if updates == 0:
        # todo what to do without js?
        return make_response("", 404)

    redirect = request.path[: -(len(f"edit/{attribute}/submit/"))]

    r = make_response("", 303)
    r.headers["Location"] = redirect
    return r


@app.route("/trip/<uuid:id>/category/<uuid:category_id>/")
def trip_with_active_category(id, category_id):
    trip = get_trip(id)

    args = request.args.to_dict()

    item_to_pick = args.pop("item_pick", None)
    if item_to_pick:
        pick_item(id, item_to_pick)
        r = make_response("", 303)
        if args:
            r.headers[
                "Location"
            ] = f"/trip/{id}/category/{category_id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/category/{category_id}/"
        return r

    item_to_unpick = args.pop("item_unpick", None)
    if item_to_unpick:
        unpick_item(id, item_to_unpick)
        r = make_response("", 303)
        if args:
            r.headers[
                "Location"
            ] = f"/trip/{id}/category/{category_id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/category/{category_id}/"
        return r

    item_to_pack = args.pop("item_pack", None)
    if item_to_pack:
        pack_item(id, item_to_pack)
        r = make_response("", 303)
        if args:
            r.headers[
                "Location"
            ] = f"/trip/{id}/category/{category_id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/category/{category_id}/"
        return r

    item_to_unpack = args.pop("item_unpack", None)
    if item_to_unpack:
        unpack_item(id, item_to_unpack)
        r = make_response("", 303)
        if args:
            r.headers[
                "Location"
            ] = f"/trip/{id}/category/{category_id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/category/{category_id}/"
        return r

    type_to_add = args.pop("type_add", None)
    if type_to_add:
        newtype = TripToTripType(trip_id=str(id), trip_type_id=str(type_to_add))
        db.session.add(newtype)
        db.session.commit()
        r = make_response("", 303)
        if args:
            r.headers[
                "Location"
            ] = f"/trip/{id}/category/{category_id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/category/{category_id}/"
        return r

    type_to_remove = args.pop("type_remove", None)
    if type_to_remove:
        newtype = TripToTripType.query.filter_by(
            trip_id=str(id), trip_type_id=str(type_to_remove)
        ).delete()
        db.session.commit()
        r = make_response("", 303)
        if args:
            r.headers[
                "Location"
            ] = f"/trip/{id}/category/{category_id}/?" + urllib.parse.urlencode(params)
        else:
            r.headers["Location"] = f"/trip/{id}/category/{category_id}/"
        return r

    items = get_all_items()

    categories = get_categories()
    active_category = [c for c in categories if str(c.id) == str(category_id)][0]
    active_category.active = True

    for item in items:
        try:
            db.session.add(
                TripItems(trip_id=str(id), item_id=str(item.id), pick=False, pack=False)
            )
            db.session.commit()
        except sqlalchemy.exc.IntegrityError:
            db.session.rollback()

    edit = args.pop("edit", None)

    return make_response(
        Base(
            TripManager(
                trip,
                categories=categories,
                active_category=active_category,
                edit=edit,
                baseurl=f"/trip/{id}/category/{category_id}/",
                triptypes=get_triptypes(),
            ),
            app.root_path,
            active_page="trips",
        ).doc.render(),
        200,
    )


@app.route("/inventory/category/<uuid:id>/")
def category(id):
    categories = get_categories()
    print(id)
    active_category = [c for c in categories if str(c.id) == str(id)][0]
    active_category.active = True

    args = request.args.to_dict()

    items = get_items(active_category)
    error = False
    if not is_htmx():
        edit = request.args.get("edit")
        if edit is not None:
            match = [i for i in items if i.id == edit]
            if match:
                match[0].edit = True

    return make_response(
        Base(
            InventoryItemManager(categories, items, active_category),
            app.root_path,
            active_page="inventory",
        ).doc.render(),
        200,
    )


def is_htmx():
    return request.headers.get("HX-Request") is not None


@app.route("/inventory/item/", methods=["POST"])
def add_new_item():
    name = request.form["name"]
    weight = int(request.form["weight"])
    category_id = request.form["category"]

    add_item(name=name, weight=weight, category_id=category_id)

    r = make_response("", 303)
    r.headers["Location"] = f"/inventory/category/{category_id}"
    return r


@app.route("/trip/", methods=["POST"])
def add_new_trip():
    name = request.form["name"]
    start_date = datetime.date.fromisoformat(request.form["start-date"])
    end_date = datetime.date.fromisoformat(request.form["end-date"])

    newid = str(uuid.uuid4())
    db.session.add(Trip(id=newid, name=name, start_date=start_date, end_date=end_date))
    db.session.commit()

    r = make_response("", 303)
    r.headers["Location"] = f"/trip/{newid}"  # TODO enable this
    r.headers["Location"] = f"/trip/"
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


@app.route("/inventory/item/<uuid:id>/edit/submit/", methods=["POST"])
def edit_item_submit(id):
    name = request.form["name"]
    weight = int(request.form["weight"])

    item = InventoryItem.query.filter_by(id=str(id)).first()
    if item is None:
        # todo what to do without js?
        return make_response("", 404)

    item.name = name
    item.weight = weight

    db.session.commit()

    r = make_response("", 303)
    r.headers["Location"] = f"/inventory/category/{item.category.id}"
    return r


@app.route("/inventory/item/<uuid:id>/pick/submit/", methods=["POST"])
def edit_item_pick(id):
    print(request.form)
    if "pick" in request.form:
        pick = request.form["pick"] == "on"
    else:
        pick = False
    print(pick)

    item = InventoryItem.query.filter_by(id=str(id)).first()
    if item is None:
        # todo what to do without js?
        return make_response("", 404)

    item.picked = pick

    db.session.commit()

    r = make_response("", 303)
    r.headers["Location"] = f"/inventory/category/{item.category.id}"
    return r


@app.route("/inventory/item/<uuid:id>/pack/submit/", methods=["POST"])
def edit_item_pack(id):
    print(request.form)
    if "pack" in request.form:
        pack = request.form["pack"] == "on"
    else:
        pack = False
    print(pack)

    item = InventoryItem.query.filter_by(id=str(id)).first()
    if item is None:
        # todo what to do without js?
        return make_response("", 404)

    item.pack = pack

    db.session.commit()

    r = make_response("", 303)
    r.headers["Location"] = f"/inventory/category/{item.category.id}"
    return r


@app.route("/inventory/item/<uuid:id>", methods=["DELETE"])
def delete_item(id):
    deletions = InventoryItem.query.filter_by(id=str(id)).delete()
    if deletions == 0:
        return make_response("", 404)
    else:
        db.session.commit()
    return make_response("", 200)


@app.route("/inventory/item/<uuid:id>/delete", methods=["GET"])
def delete_item_get(id):
    print(request.headers)
    print(request.args)
    print(f"deleting {id}")
    deletions = InventoryItem.query.filter_by(id=str(id)).delete()
    if deletions == 0:
        return make_response("", 404)
    else:
        db.session.commit()
    r = make_response("", 303)
    r.headers["Location"] = request.headers["Referer"]
    return r
