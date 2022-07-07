#!/usr/bin/env bash

xtightvncviewer 127.0.0.1::5900 -passwd <(printf %s secret | vncpasswd -f)
