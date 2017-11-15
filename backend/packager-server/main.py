#!/usr/bin/env python3
from flask import Flask, jsonify

import sqlite3

app = Flask(__name__)

def get_lists():
    connection = sqlite3.connect('../data/example.db')
    cursor = connection.cursor()

    print(list(cursor.execute('''SELECT * FROM lists''')))

    lists = [
        "list_item_1",
        "list_item_2",
        "list_item_3",
        "this shiny new item",
        "more stuff"
    ]

    connection.commit()
    connection.close()

    return lists

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
