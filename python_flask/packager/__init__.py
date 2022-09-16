import uuid
import sqlalchemy
import csv
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
    categories = (
        {"id": uuid.uuid4(), "name": "Sleeping"},
        {"id": uuid.uuid4(), "name": "Shelter"},
        {"id": uuid.uuid4(), "name": "Fire"},
        {"id": uuid.uuid4(), "name": "Cooking"},
        {"id": uuid.uuid4(), "name": "Water"},
        {"id": uuid.uuid4(), "name": "Protection"},
        {"id": uuid.uuid4(), "name": "Tools"},
        {"id": uuid.uuid4(), "name": "Insulation"},
        {"id": uuid.uuid4(), "name": "Electronics"},
        {"id": uuid.uuid4(), "name": "Carry"},
        {"id": uuid.uuid4(), "name": "Medic"},
        {"id": uuid.uuid4(), "name": "Hygiene"},
    )

    for category in categories:
        db.session.add(
            PackageListItemCategory(
                id=str(category['id']),
                name=category['name'],
                description="",
            )
        )

    with open("./items.csv") as csvfile:
        reader = csv.reader(csvfile, delimiter=',')
        for row in reader:
            print(row)
            (name, category, weight) = row

            db.session.add(
                PackageListItem(
                    id=str(uuid.uuid4()),
                    name=name,
                    description="",
                    weight=weight,
                    category_id=str([c['id'] for c in categories if c['name'] == category][0])
                )
            )


    print("db init done")
    db.session.commit()
except sqlalchemy.exc.IntegrityError:
    pass
