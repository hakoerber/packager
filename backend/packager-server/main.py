#!/usr/bin/env python3
from flask import Flask, jsonify

app = Flask(__name__)

def get_lists():
    return [
        "list_item_1",
        "list_item_2",
        "list_item_3",
    ]

@app.route('/')
def index():
    return "Hello, World!"

@app.route('/api/')
def api():
    return "Hello, API!"

@app.route('/api/v1/')
def apiv1():
    return "Hello, API version 1!"

@app.route('/api/v1/lists/')
def apiv1_lists():
    return jsonify(get_lists())

if __name__ == '__main__':
    app.run(debug=True, port=8000)
