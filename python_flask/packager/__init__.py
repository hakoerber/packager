import uuid
import sqlalchemy
import csv
from flask import Flask
from flask_migrate import Migrate

from .helpers import *

from flask_sqlalchemy import SQLAlchemy

app = Flask(__name__)
app.config["SQLALCHEMY_DATABASE_URI"] = f"sqlite:///{app.root_path}/../db.sqlite"
app.config["SQLALCHEMY_TRACK_MODIFICATIONS"] = False

db = SQLAlchemy(app)
migrate = Migrate(app, db, render_as_batch=True)

from packager.models import *
import packager.views

db.create_all()
