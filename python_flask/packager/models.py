from . import db
import enum


class InventoryItemCategory(db.Model):
    __tablename__ = "inventoryitemcategories"
    id = db.Column(db.String(36), primary_key=True)
    name = db.Column(db.Text, unique=True, nullable=False)
    description = db.Column(db.Text)
    items = db.relationship("InventoryItem", backref="category", lazy=True)

    active = False


class InventoryItem(db.Model):
    __tablename__ = "inventoryitems"
    id = db.Column(db.String(36), primary_key=True)
    name = db.Column(db.Text, unique=True, nullable=False)
    description = db.Column(db.Text)
    weight = db.Column(db.Integer, nullable=False)
    category_id = db.Column(
        db.String(36), db.ForeignKey("inventoryitemcategories.id"), nullable=False
    )

    edit = False


class TripItems(db.Model):
    __tablename__ = "tripitems"
    item_id = db.Column(
        db.String(36),
        db.ForeignKey("inventoryitems.id"),
        nullable=False,
        primary_key=True,
    )
    trip_id = db.Column(
        db.String(36), db.ForeignKey("trips.id"), nullable=False, primary_key=True
    )
    inventory_item = db.relationship("InventoryItem", lazy=True)

    pick = db.Column(db.Boolean, nullable=False)
    pack = db.Column(db.Boolean, nullable=False)

    edit = False


class TripType(db.Model):
    __tablename__ = "triptypes"
    id = db.Column(db.String(36), primary_key=True)
    name = db.Column(db.Text, unique=True, nullable=False)


class TripToTripType(db.Model):
    __tablename__ = "trips_to_triptypes"
    trip_id = db.Column(
        db.String(36), db.ForeignKey("trips.id"), nullable=False, primary_key=True
    )
    trip_type_id = db.Column(
        db.String(36), db.ForeignKey("triptypes.id"), nullable=False, primary_key=True
    )


class TripState(enum.Enum):
    Planning = 1
    Planned = 2
    Active = 3
    Review = 4
    Done = 5


class Trip(db.Model):
    __tablename__ = "trips"
    id = db.Column(db.String(36), primary_key=True)
    name = db.Column(db.Text, unique=True, nullable=False)
    start_date = db.Column(db.Date, nullable=False)
    end_date = db.Column(db.Date, nullable=False)
    location = db.Column(db.Text, nullable=False)
    temp_min = db.Column(db.Integer, nullable=False)
    temp_max = db.Column(db.Integer, nullable=False)

    comment = db.Column(db.Text, nullable=False)

    types = db.relationship("TripType", secondary="trips_to_triptypes", lazy=True)
    items = db.relationship("TripItems", lazy=True)

    state = db.Column(db.Enum(TripState), nullable=False)
