#!/usr/bin/env python3
from flask import Flask, jsonify, abort, request

import sqlite3

app = Flask(__name__)

lists = None

def get_db_connection():
    connection = sqlite3.connect('../data/example.db')
    cursor = connection.cursor()
    return (connection, cursor)

def db_close(connection, cursor):
    cursor.close()
    connection.commit()
    connection.close()

def _load_lists():
    connection, cursor = get_db_connection()

    cursor.execute('''SELECT * FROM lists''')
    raw = cursor.fetchall()
    lists = [
        {
            'id': raw[i][0],
            'name': raw[i][1],
        } for i in range(len(raw))]

    db_close(connection, cursor)
    return lists

def get_lists():
    global lists
    if not lists:
        lists = _load_lists()
    print(lists)
    return lists

def reload_lists():
    global lists
    lists = _load_lists()

def get_list_by_id(list_id):
    pkglist = [i for i in get_lists() if i['id'] == list_id]
    if pkglist:
        return pkglist[0]
    return None

def add_new_list(name):
    connection, cursor = get_db_connection()
    cursor.execute('''INSERT INTO lists (id, name) VALUES (NULL, ?)''', (name,))
    db_close(connection, cursor)
    reload_lists()

@app.route('/')
def index():
    return "Hello, World!"

@app.route('/api/')
def api():
    return "Hello, API!"

@app.route('/api/v1/')
def apiv1():
    return "Hello, API version 1!"

@app.route('/api/v1/lists/', methods=('GET', 'POST'))
def apiv1_lists():
    if request.method == 'POST':
        payload = request.get_json()
        add_new_list(payload['name'])
        return "success"
    else:
        return jsonify(get_lists())


@app.route('/api/v1/lists/<int:list_id>/')
def apiv1_list_by_id(list_id):
    pkglist = get_list_by_id(list_id)
    print(pkglist)
    if pkglist:
        return jsonify(pkglist)
    else:
        return ("id not found", 404)



if __name__ == '__main__':
    app.run(debug=True, port=8000)
