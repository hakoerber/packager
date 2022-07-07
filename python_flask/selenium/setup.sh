#!/usr/bin/env bash

set -o errexit
set -o nounset
set -p pipefail

python3 -m venv ./venv
source ./venv/bin/activate
python3 -m pip install -r requirements.txt

sudo apt install tigervnc-common xtightvncviewer
