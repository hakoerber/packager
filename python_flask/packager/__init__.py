import uuid
import sqlalchemy
from flask import Flask

from .helpers import *

from flask_sqlalchemy import SQLAlchemy

app = Flask(__name__)
app.config["SQLALCHEMY_DATABASE_URI"] = f"sqlite:///{app.root_path}/../db.sqlite"
app.config["SQLALCHEMY_TRACK_MODIFICATIONS"] = False
db = SQLAlchemy(app)

from packager.models import *
import packager.views


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

    db.session.add(
        PackageListItem(
            id="4c08f0d5-583e-4882-8bea-2b2faab61fff",
            name="Taschenmesser",
            description="",
            packagelist_id="ab2f16c2-d5f5-460b-b149-0fc9eec12887",
        )
    )

    db.session.add(
        PackageListItem(
            id="f7fe1c35-23c8-4e57-bec0-56212cff940a",
            name="Geldbeutel",
            description="",
            packagelist_id="ab2f16c2-d5f5-460b-b149-0fc9eec12887",
        )
    )

    db.session.commit()
except sqlalchemy.exc.IntegrityError:
    pass
