#!/usr/bin/env bash

source ./venv/bin/activate

export FLASK_APP=packager
export FLASK_ENV=development

if (( $# == 0 )) ; then
    python3 -m flask run --reload --host 0.0.0.0 --port 5000
else
    python3 -m flask "${@}"
fi
