from . import db


class PackageList(db.Model):
    __tablename__ = "packagelist"
    id = db.Column(db.String(36), primary_key=True)
    name = db.Column(db.Text, unique=True, nullable=False)
    description = db.Column(db.Text)
    items = db.relationship("PackageListItem", backref="packagelist", lazy=True)

    edit = False
    error = False
    errormsg = None


class PackageListItem(db.Model):
    __tablename__ = "packagelistitem"
    id = db.Column(db.String(36), primary_key=True)
    name = db.Column(db.Text, unique=True, nullable=False)
    description = db.Column(db.Text)
    packagelist_id = db.Column(
        db.String(36), db.ForeignKey("packagelist.id"), nullable=False
    )
