#!/usr/bin/env python3
from flask import Flask, jsonify

import sqlite3

app = Flask(__name__)

def get_lists():
    connection = sqlite3.connect('../data/example.db')
    cursor = connection.cursor()

    cursor.execute('''SELECT * FROM lists''')
    raw = cursor.fetchall()
    lists = [
        {
            'id': i,
            'name': raw[i][0],
        } for i in range(len(raw))]

    cursor.close()
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
