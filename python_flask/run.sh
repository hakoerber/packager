#!/usr/bin/env bash

source ./venv/bin/activate

export FLASK_APP=packager
export FLASK_ENV=development

python3 -m flask run --reload
